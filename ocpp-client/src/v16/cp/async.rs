use core::task::Poll;

use alloc::vec::Vec;
use futures::FutureExt;

use crate::v16::{services::{database::DatabaseService, diagnostics::DiagnosticsService, firmware::{FirmwareResponse, FirmwareService}, hardware::HardwareService, secc::SeccService, stop::StopChargePointService, timeout::TimeoutService, websocket::{WebsocketResponse, WebsocketService}}, state_machine::actions::CoreActions, Database, Diagnostics, Firmware, Hardware, HardwareActions, Secc, StopChargePoint, Timeout, WebsocketIo};

use super::{config::ChargePointConfig, ChargePointCore};

pub struct ChargePointAsync<DB: Database, WS: WebsocketIo, DI: Diagnostics, FW: Firmware, SE: Secc, TI: Timeout, HW: Hardware, ST: StopChargePoint> {
    configs: ChargePointConfig,
    ws: WebsocketService<WS>,
    diagnostics: DiagnosticsService<DI>,
    firmware: FirmwareService<FW>,
    db: DatabaseService<DB>,
    secc: SeccService<SE>,
    timeout: TimeoutService<TI>,
    hardware: HardwareService<HW>,
    stop: StopChargePointService<ST>
}

impl<DB: Database, WS: WebsocketIo, DI: Diagnostics, FW: Firmware, SE: Secc, TI: Timeout, HW: Hardware, ST: StopChargePoint>
    ChargePointAsync<DB, WS, DI, FW, SE, TI, HW, ST>
{
    pub fn new(
        ws: WS,
        diagnostics: DI,
        firmware: FW,
        db: DB,
        secc: SE,
        timeout: TI,
        hardware: HW,
        stop: ST,
        configs: ChargePointConfig,
    ) -> Self {
        Self {
            configs,
            ws: WebsocketService::new(ws),
            diagnostics: DiagnosticsService::new(diagnostics),
            firmware: FirmwareService::new(firmware),
            db: DatabaseService::new(db),
            secc: SeccService::new(secc),
            timeout: TimeoutService::new(timeout),
            hardware: HardwareService::new(hardware),
            stop: StopChargePointService::new(stop)
        }
    }
    async fn run_once(self) -> (Self, bool) {
        let ChargePointAsync {
            mut ws,
            mut diagnostics,
            mut firmware,
            db,
            secc,
            mut timeout,
            mut hardware,
            mut stop,
            mut configs,
        } = self;
        configs.seed = configs.seed.saturating_add(1);
        let mut cp = ChargePointCore::new(
            db,
            secc,
            configs.clone()
        );
        cp.init();
        let mut soft_reset = false;
        'main: loop {
            let (actions, stop) = futures::future::poll_fn(|cx| {
                if let Poll::Ready(_) = stop.poll_unpin(cx) {
                    return Poll::Ready((Vec::new(), true));
                }
                if let Poll::Ready(event) = hardware.poll_unpin(cx) {
                    log::info!("got event {:?}", event);
                    let actions = match event {
                        HardwareActions::State(connector_id, state, error_code, info) => {
                            cp.secc_change_state(connector_id, state, error_code, info)
                        },
                        HardwareActions::IdTag(connector_id, id_tag) => {
                            cp.secc_id_tag(connector_id, id_tag)
                        }
                    };
                    return Poll::Ready((actions, false));
                }
                if let Poll::Ready(msg) = ws.poll_unpin(cx) {
                    let actions = match msg {
                        WebsocketResponse::Connected => {
                            log::info!("ws connected");
                            cp.ws_connected()
                        },
                        WebsocketResponse::Disconnected => {
                            log::info!("ws disconnected");
                            cp.ws_disconnected()
                        },
                        WebsocketResponse::WsMsg(msg) => {
                            log::info!("[MSG_IN] {}", msg);
                            cp.got_ws_msg(msg)
                        }
                    };
                    return Poll::Ready((actions, false));
                }
                if let Poll::Ready(id) = timeout.poll_unpin(cx) {
                    log::trace!("id timedout: {:?}", id);
                    let actions = cp.handle_timeout(id);
                    return Poll::Ready((actions, false));
                }
                if let Poll::Ready(msg) = firmware.poll_unpin(cx) {
                    log::debug!("received firmware event: {:?}", msg);
                    let actions = match msg {
                        FirmwareResponse::DownloadStatus(res) => {
                            cp.firmware_download_response(res)
                        },
                        FirmwareResponse::InstallStatus(res) => {
                            cp.firmware_install_response(res)
                        }
                    };
                    return Poll::Ready((actions, false));
                }
                if let Poll::Ready(msg) = diagnostics.poll_unpin(cx) {
                    log::debug!("received diagnostic event: {:?}", msg);
                    let actions = cp.handle_diagnostics_response(msg);
                    return Poll::Ready((actions, false));
                }
                Poll::Pending
            }).await;
            if stop {
                break;
            }
            for action in actions {
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
                        timeout
                    } => {
                        log::debug!("start upload, location: {},file_name: {}, start_time: {:?}, stop_time: {:?}, timeout: {:?}", location, file_name, start_time, stop_time, timeout);
                        diagnostics.start_upload(location, file_name, start_time, stop_time, timeout);
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
                            deadline
                        );
                        timeout.add_or_update(id, deadline);
                    }
                    CoreActions::RemoveTimeout(id) => {
                        log::trace!("remove timeout, id: {:?}", id);
                        timeout.remove_timeout(id);
                    }
                    CoreActions::SoftReset => {
                        log::warn!("soft reset");
                        diagnostics.make_idle().await;
                        firmware.make_idle().await;
                        timeout.remove_all_timeouts();
                        ws.close_connection().await;
                        cp.pending_reset = None;

                        // if want to rebuild CP again after reset (will send bootnotification after soft reset)
                        soft_reset = true;
                        break 'main;
                    }
                }
            }
        }
        let ChargePointCore { db, secc, .. } = cp;
        (Self {
            configs,
            ws,
            diagnostics,
            firmware,
            db,
            secc,
            timeout,
            hardware,
            stop
        }, soft_reset)
    }
    pub async fn run(mut self) {
        self.db.db_init(self.configs.default_ocpp_configs.clone(), self.configs.clear_db);
        loop {
            let (t, soft_reset) = self.run_once().await;
            if !soft_reset {
                break;
            }
            self = t;
        }
    }
}
