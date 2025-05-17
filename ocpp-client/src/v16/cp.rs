use std::{collections::HashMap, time::Instant};

use flume::Receiver;
use futures::StreamExt;
use ocpp_core::v16::messages::boot_notification::BootNotificationRequest;
use tokio_util::sync::CancellationToken;

use crate::v16::{
    services::{firmware::FirmwareResponse, websocket::WebsocketResponse},
    state_machine::actions::CoreActions,
};

use super::{
    interface::{Database, Diagnostics, Firmware, Secc, WebsocketIo},
    services::{
        database::DatabaseService,
        diagnostics::DiagnosticsService,
        firmware::FirmwareService,
        secc::{SeccActions, SeccService},
        timeout::TimeoutService,
        websocket::WebsocketService,
    },
    state_machine::core::ChargePointCore,
};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ChargePointConfig {
    pub cms_url: String,
    pub call_timeout: u64,
    pub max_cache_len: usize,
    pub boot_info: BootNotificationRequest,
    pub default_ocpp_configs: HashMap<String, String>,
    pub clear_db: bool,
}

pub struct ChargePoint<DB: Database, WS: WebsocketIo, DI: Diagnostics, FW: Firmware, S: Secc> {
    config: ChargePointConfig,
    ws: WebsocketService<WS>,
    diagnostics: DiagnosticsService<DI>,
    firmware: FirmwareService<FW>,
    db: DatabaseService<DB>,
    secc: SeccService<S>,
    timeout: TimeoutService,
    secc_rx: Receiver<SeccActions>,
    stop_token: CancellationToken,
}

impl<DB: Database, WS: WebsocketIo, DI: Diagnostics, FW: Firmware, S: Secc>
    ChargePoint<DB, WS, DI, FW, S>
{
    pub fn new(
        ws: WS,
        diagnostics: DI,
        firmware: FW,
        db: DB,
        secc: S,
        config: ChargePointConfig,
        stop_token: CancellationToken,
        secc_rx: Receiver<SeccActions>,
    ) -> Self {
        let configs = config.default_ocpp_configs.clone();
        let clear_db = config.clear_db;

        let mut db = DatabaseService::new(db);
        db.db_init(configs, clear_db);

        Self {
            config,
            ws: WebsocketService::new(ws),
            diagnostics: DiagnosticsService::new(diagnostics),
            firmware: FirmwareService::new(firmware),
            db,
            secc: SeccService::new(secc),
            timeout: TimeoutService::new(),
            secc_rx,
            stop_token,
        }
    }
    async fn run_once(self) -> Self {
        let ChargePoint {
            mut ws,
            mut diagnostics,
            mut firmware,
            db,
            secc,
            mut timeout,
            secc_rx,
            config,
            stop_token,
        } = self;
        let mut cp = ChargePointCore::new(
            db,
            secc,
            config.cms_url.clone(),
            config.call_timeout,
            config.max_cache_len,
            config.boot_info.clone(),
        );
        cp.init();
        'main: loop {
            for action in cp.queued_actions.drain(..) {
                match action {
                    CoreActions::Connect(cms_url) => {
                        log::debug!("connect, cms_url: {}", cms_url);
                        ws.connect(cms_url);
                    }
                    CoreActions::SendWsMsg(msg) => {
                        log::info!("[MSG_OUT] {}", msg);
                        ws.send_msg(msg).await;
                    }
                    CoreActions::StartDiagnosticUpload {
                        location,
                        file_name,
                        start_time,
                        stop_time,
                    } => {
                        log::debug!("start upload, location: {},file_name: {}, start_time: {:?}, stop_time: {:?}", location, file_name, start_time, stop_time);
                        diagnostics.start_upload(location, file_name, start_time, stop_time);
                    }
                    CoreActions::CancelDiagnosticUpload => {
                        log::debug!("cancel diagnostics");
                        diagnostics.cancel_upload();
                    }
                    CoreActions::DownloadFirmware(location) => {
                        log::debug!("download firmware, location: {}", location);
                        firmware.start_firmware_download(location);
                    }
                    CoreActions::InstallFirmware(firmware_image) => {
                        log::debug!("install firmware, {:?}", firmware_image);
                        firmware.start_firmware_install(firmware_image);
                    }
                    CoreActions::AddTimeout(id, deadline) => {
                        log::trace!(
                            "add timeout, id: {:?}, deadline: {:?}",
                            id,
                            deadline.saturating_duration_since(Instant::now())
                        );
                        timeout.add_or_update(id, deadline);
                    }
                    CoreActions::RemoveTimeout(id) => {
                        log::trace!("remove timeout, id: {:?}", id);
                        timeout.remove(id);
                    }
                    CoreActions::SoftReset => {
                        log::warn!("soft reset");
                        diagnostics.make_idle().await;
                        firmware.make_idle().await;
                        timeout.remove_all();
                        ws.close_connection().await;
                        cp.pending_reset = None;

                        // if want to rebuild CP again after reset (will send bootnotification after soft reset)
                        break 'main;
                    }
                }
            }
            tokio::select! {
                biased;
                _ = stop_token.cancelled() => {
                    break;
                }
                msg = secc_rx.recv_async() => {
                    let msg = msg.unwrap();
                    log::info!("received secc msg: {:?}", msg);
                    match msg {
                        SeccActions::Secc(connector_id, state, error_code, info) => {
                            cp.secc_change_state(connector_id, state, error_code, info);
                        },
                        SeccActions::IdTag(connector_id, id_tag) => {
                            cp.secc_id_tag(connector_id, id_tag);
                        }
                    }
                }

                msg = &mut ws => {
                    match msg {
                        WebsocketResponse::Connected => {
                            log::info!("ws connected");
                            cp.connected();
                        },
                        WebsocketResponse::Disconnected => {
                            log::info!("ws disconnected");
                            cp.disconnected();
                        },
                        WebsocketResponse::WsMsg(msg) => {
                            log::info!("[MSG_IN] {}", msg);
                            cp.got_ws_msg(msg);
                        }
                    }
                }

                id = timeout.next() => {
                    log::trace!("id timedout: {:?}", id);
                    if let Some(id) = id {
                        cp.handle_timeout(id);
                    }
                }
                msg = &mut firmware => {
                    log::debug!("received firmware event: {:?}", msg);
                    match msg {
                        FirmwareResponse::DownloadStatus(res) => {
                            cp.firmware_download_response(res);
                        },
                        FirmwareResponse::InstallStatus(res) => {
                            cp.firmware_install_response(res);
                        }
                    }
                }
                msg = &mut diagnostics => {
                    log::debug!("received diagnostic event: {:?}", msg);
                    cp.handle_diagnostics_response(msg);
                }
            }
        }
        let ChargePointCore { db, secc, .. } = cp;
        Self {
            config,
            ws,
            diagnostics,
            firmware,
            db,
            secc,
            secc_rx,
            timeout,
            stop_token,
        }
    }
    pub async fn run(mut self) {
        loop {
            let fut = tokio::spawn(self.run_once());
            match fut.await {
                Ok(t) => {
                    if t.stop_token.is_cancelled() {
                        log::debug!("stop_token cancelled");
                        break;
                    }
                    self = t;
                }
                Err(e) => {
                    log::error!("Crashed {:?}", e);
                    break;
                }
            }
        }
    }
}
