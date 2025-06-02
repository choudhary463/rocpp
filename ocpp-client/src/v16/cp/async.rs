use core::task::Poll;

use alloc::vec::Vec;
use futures::FutureExt;

use crate::v16::{drivers::{database::{ChargePointStorage, Database}, diagnostics::{Diagnostics, DiagnosticsManager}, firmware::{Firmware, FirmwareManager, FirmwareResponse}, hardware_interface::{HardwareBridge, HardwareInterface}, peripheral_input::{PeripheralActions, PeripheralDriver, PeripheralInput}, shutdown::{ShutdownDriver, ShutdownSignal}, timers::{TimerDriver, TimerManager}, websocket::{WebsocketClient, WebsocketResponse, WebsocketTransport}}, state_machine::actions::CoreActions};

use super::{config::ChargePointConfig, core::ChargePointCore};

pub struct ChargePointAsync<DB: Database, WS: WebsocketTransport, DI: Diagnostics, FW: Firmware, HW: HardwareInterface, TI: TimerManager, PI: PeripheralInput, ST: ShutdownSignal> {
    configs: ChargePointConfig,
    ws: WebsocketClient<WS>,
    diagnostics: DiagnosticsManager<DI>,
    firmware: FirmwareManager<FW>,
    db: ChargePointStorage<DB>,
    hw: HardwareBridge<HW>,
    timer: TimerDriver<TI>,
    peripheral: PeripheralDriver<PI>,
    shutdown: ShutdownDriver<ST>
}

impl<DB: Database, WS: WebsocketTransport, DI: Diagnostics, FW: Firmware, HW: HardwareInterface, TI: TimerManager, PI: PeripheralInput, ST: ShutdownSignal>
    ChargePointAsync<DB, WS, DI, FW, HW, TI, PI, ST>
{
    pub fn new(
        ws: WS,
        diagnostics: DI,
        firmware: FW,
        db: DB,
        hw: HW,
        timer: TI,
        peripheral: PI,
        shutdown: ST,
        configs: ChargePointConfig,
    ) -> Self {
        Self {
            configs,
            ws: WebsocketClient::new(ws),
            diagnostics: DiagnosticsManager::new(diagnostics),
            firmware: FirmwareManager::new(firmware),
            db: ChargePointStorage::new(db),
            hw: HardwareBridge::new(hw),
            timer: TimerDriver::new(timer),
            peripheral: PeripheralDriver::new(peripheral),
            shutdown: ShutdownDriver::new(shutdown)
        }
    }
    async fn run_once(self) -> (Self, bool) {
        let ChargePointAsync {
            mut ws,
            mut diagnostics,
            mut firmware,
            db,
            hw,
            mut timer,
            mut peripheral,
            mut shutdown,
            mut configs,
        } = self;
        configs.seed = configs.seed.saturating_add(1);
        let mut cp = ChargePointCore::new(
            db,
            hw,
            configs.clone()
        );
        cp.init();
        let mut soft_reset = false;
        'main: loop {
            let (actions, shutdown) = futures::future::poll_fn(|cx| {
                if let Poll::Ready(_) = shutdown.poll_unpin(cx) {
                    return Poll::Ready((Vec::new(), true));
                }
                if let Poll::Ready(action) = peripheral.poll_unpin(cx) {
                    log::info!("got action {:?}", action);
                    let actions = match action {
                        PeripheralActions::State(connector_id, state, error_code, info) => {
                            cp.secc_change_state(connector_id, state, error_code, info)
                        },
                        PeripheralActions::IdTag(connector_id, id_tag) => {
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
                if let Poll::Ready(id) = timer.poll_unpin(cx) {
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
            if shutdown {
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
                        timer.add_or_update(id, deadline);
                    }
                    CoreActions::RemoveTimeout(id) => {
                        log::trace!("remove timeout, id: {:?}", id);
                        timer.remove_timeout(id);
                    }
                    CoreActions::SoftReset => {
                        log::warn!("soft reset");
                        diagnostics.make_idle().await;
                        firmware.make_idle().await;
                        timer.remove_all_timeouts();
                        ws.close_connection().await;
                        cp.pending_reset = None;

                        // if want to rebuild CP again after reset (will send bootnotification after soft reset)
                        soft_reset = true;
                        break 'main;
                    }
                }
            }
        }
        let ChargePointCore { db, hw, .. } = cp;
        (Self {
            configs,
            ws,
            diagnostics,
            firmware,
            db,
            hw,
            timer,
            peripheral,
            shutdown
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
