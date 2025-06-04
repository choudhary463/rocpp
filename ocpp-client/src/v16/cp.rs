use core::{future::poll_fn, task::Poll};

use alloc::{
    collections::{btree_map::BTreeMap, vec_deque::VecDeque},
    string::String,
    vec,
    vec::Vec,
};
use chrono::{DateTime, Utc};
use rand::{rngs::SmallRng, SeedableRng};
use rocpp_core::{
    format::frame::Call,
    v16::{
        messages::{
            boot_notification::BootNotificationRequest,
            status_notification::StatusNotificationRequest,
        },
        protocol_error::ProtocolError,
        types::{RegistrationStatus, ResetType},
    },
};

use crate::v16::state_machine::{
    boot::BootState,
    call::{CallAction, OutgoingCallState},
    clock::Instant,
    config::OcppConfigs,
    connector::{ConnectorState, StatusNotificationState},
    diagnostics::DiagnosticsState,
    firmware::FirmwareState,
    heartbeat::HeartbeatState,
    meter::MeterState,
    transaction::TransactionEventState,
};

use super::{
    interfaces::{
        ChargePointBackend, ChargePointEvent, ChargePointInterface, HardwareEvent, WsEvent,
    },
    state_machine::transaction::TransactionEvent,
};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ChargePointConfig {
    pub cms_url: String,
    pub call_timeout: u64,
    pub boot_info: BootNotificationRequest,
    pub default_ocpp_configs: Vec<(String, String)>,
    pub clear_db: bool,
    pub seed: u64,
}

pub(crate) type OcppError = rocpp_core::format::error::OcppError<ProtocolError>;

pub struct ChargePoint<I: ChargePointInterface> {
    pub(crate) interface: ChargePointBackend<I>,
    pub(crate) rng: SmallRng,
    pub(crate) cms_url: String,
    pub(crate) boot_info: BootNotificationRequest,
    pub(crate) ws_connected: bool,
    pub(crate) call_timeout: u64,
    pub(crate) outgoing_call_state: OutgoingCallState,
    pub(crate) pending_calls: VecDeque<(Call, CallAction)>,
    pub(crate) boot_state: BootState,
    pub(crate) registration_status: RegistrationStatus,
    pub(crate) heartbeat_state: HeartbeatState,
    pub(crate) base_time: Option<(DateTime<Utc>, Instant)>,
    pub(crate) pending_auth_requests: VecDeque<(usize, String)>,
    pub(crate) local_list_entries_count: usize,
    pub(crate) connector_state: Vec<ConnectorState>,
    pub(crate) connector_status_notification: Vec<StatusNotificationRequest>,
    pub(crate) connector_status_notification_state: Vec<StatusNotificationState>,
    pub(crate) pending_inoperative_changes: Vec<bool>,
    pub(crate) sampled_meter_state: Vec<MeterState>,
    pub(crate) aligned_meter_state: MeterState,
    pub(crate) local_transaction_id: u32,
    pub(crate) active_local_transactions: Vec<Option<(u32, Option<i32>)>>,
    pub(crate) transaction_head: u64,
    pub(crate) transaction_tail: u64,
    pub(crate) transaction_map: BTreeMap<u32, i32>,
    pub(crate) transaction_connector_map: BTreeMap<u32, usize>,
    pub(crate) transaction_stop_meter_val_count: BTreeMap<u32, usize>,
    pub(crate) transaction_event_state: TransactionEventState,
    pub(crate) transaction_event_retries: u64,
    pub(crate) transacion_current_event: Option<TransactionEvent>,
    pub(crate) diagnostics_state: DiagnosticsState,
    pub(crate) firmware_state: FirmwareState,
    pub(crate) pending_reset: Option<ResetType>,
    pub(crate) soft_reset_now: bool,
    pub(crate) configs: OcppConfigs,
}

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn new(
        mut interface: ChargePointBackend<I>,
        configs: ChargePointConfig,
    ) -> Self {
        let db_configs = interface.db_get_all_configs().await;

        let ocpp_configs = OcppConfigs::build(db_configs);

        let num_connectors = ocpp_configs.number_of_connectors.value;

        let (connector_state, connector_status_notification) =
            interface.db_get_connector_state(num_connectors).await;

        let (
            local_transaction_id,
            transaction_tail,
            transaction_head,
            transaction_map,
            transaction_connector_map,
            transaction_stop_meter_val_count,
            unfinished_transactions,
        ) = interface.db_get_transaction_data().await;

        let local_list_entries_count = interface.db_get_local_list_entries_count().await;
        let mut res = Self {
            interface,
            rng: SmallRng::seed_from_u64(configs.seed),
            cms_url: configs.cms_url,
            boot_info: configs.boot_info,
            ws_connected: false,
            call_timeout: configs.call_timeout,
            outgoing_call_state: OutgoingCallState::Idle,
            pending_calls: VecDeque::new(),
            boot_state: BootState::Idle,
            registration_status: RegistrationStatus::Rejected,
            heartbeat_state: HeartbeatState::Idle,
            base_time: None,
            local_list_entries_count,
            pending_auth_requests: VecDeque::new(),
            connector_state,
            connector_status_notification,
            connector_status_notification_state: vec![
                StatusNotificationState::Offline(None);
                num_connectors
            ],
            pending_inoperative_changes: vec![false; num_connectors],
            sampled_meter_state: vec![MeterState::Idle; num_connectors],
            aligned_meter_state: MeterState::Idle,
            local_transaction_id,
            active_local_transactions: vec![None; num_connectors],
            transaction_head,
            transaction_tail,
            transaction_map,
            transaction_connector_map,
            transaction_stop_meter_val_count,
            transaction_event_state: TransactionEventState::Idle,
            transaction_event_retries: 0,
            transacion_current_event: None,
            diagnostics_state: DiagnosticsState::Idle,
            firmware_state: FirmwareState::Idle,
            pending_reset: None,
            soft_reset_now: false,
            configs: ocpp_configs,
        };
        res.handle_unfinished_transactions(unfinished_transactions)
            .await;
        res
    }

    async fn run_once(
        interface: ChargePointBackend<I>,
        configs: ChargePointConfig,
    ) -> (ChargePointBackend<I>, bool) {
        let mut cp = Self::new(interface, configs).await;
        cp.init().await;
        let mut soft_reset = false;
        loop {
            let event = poll_fn(|cx| {
                if let Poll::Ready(_) = cp.interface.interface.poll_reset(cx) {
                    return Poll::Ready(ChargePointEvent::Reset);
                }
                if let Poll::Ready(t) = cp.interface.interface.poll_hardware_events(cx) {
                    return Poll::Ready(ChargePointEvent::Hardware(t));
                }
                if let Poll::Ready(t) = cp.interface.interface.poll_ws_recv(cx) {
                    return Poll::Ready(ChargePointEvent::Ws(t));
                }
                if let Poll::Ready(id) = cp.interface.interface.poll_timeout(cx) {
                    return Poll::Ready(ChargePointEvent::Timeout(id));
                }
                match &cp.firmware_state {
                    FirmwareState::Downloading(_) => {
                        if let Poll::Ready(res) = cp.interface.interface.poll_firmware_download(cx)
                        {
                            return Poll::Ready(ChargePointEvent::FirmwareDownload(res));
                        }
                    }
                    FirmwareState::Installing => {
                        if let Poll::Ready(res) = cp.interface.interface.poll_firmware_install(cx) {
                            return Poll::Ready(ChargePointEvent::FirmwareInstall(res));
                        }
                    }
                    _ => {}
                }
                match &cp.diagnostics_state {
                    DiagnosticsState::Uploading(_) => {
                        if let Poll::Ready(res) = cp.interface.interface.poll_diagnostics_upload(cx)
                        {
                            return Poll::Ready(ChargePointEvent::Diagnostics(res));
                        }
                    }
                    _ => {}
                }
                Poll::Pending
            })
            .await;
            match event {
                ChargePointEvent::Reset => {
                    log::info!("reset trigger");
                    break;
                }
                ChargePointEvent::Hardware(ev) => {
                    log::info!("received hardware event: {:?}", ev);
                    match ev {
                        HardwareEvent::IdTag(connector_id, id_tag) => {
                            cp.secc_id_tag(connector_id, id_tag).await;
                        }
                        HardwareEvent::State(connector_id, state, error_code, info) => {
                            cp.secc_change_state(connector_id, state, error_code, info)
                                .await;
                        }
                    }
                }
                ChargePointEvent::Ws(ev) => match ev {
                    WsEvent::Connected => {
                        log::info!("ws connected");
                        assert!(!cp.ws_connected);
                        cp.ws_connected().await;
                    }
                    WsEvent::Disconnected => {
                        log::info!("ws disconnected");
                        assert!(cp.ws_connected);
                        cp.ws_disconnected().await;
                    }
                    WsEvent::Msg(msg) => {
                        log::info!("[MSG_IN] {}", msg);
                        assert!(cp.ws_connected);
                        cp.got_ws_msg(msg).await;
                    }
                },
                ChargePointEvent::Timeout(id) => {
                    log::trace!("id timedout: {:?}", id);
                    cp.handle_timeout(id).await;
                }
                ChargePointEvent::FirmwareDownload(res) => {
                    log::debug!("firmware download res: {}", res);
                    cp.firmware_download_response(res).await;
                }
                ChargePointEvent::FirmwareInstall(res) => {
                    log::debug!("firmware install res: {}", res);
                    cp.firmware_install_response(res).await;
                }
                ChargePointEvent::Diagnostics(res) => {
                    log::debug!("diagnostics upload res: {:?}", res);
                    cp.handle_diagnostics_response(res).await;
                }
            }
            if cp.soft_reset_now {
                cp.interface.interface.ws_close().await;
                cp.interface.interface.remove_all_timeouts().await;
                loop {
                    let res = poll_fn(|cx| cp.interface.interface.poll_ws_recv(cx)).await;
                    if res == WsEvent::Disconnected {
                        break;
                    }
                }
                match &cp.firmware_state {
                    FirmwareState::Downloading(_) => {
                        let res =
                            poll_fn(|cx| cp.interface.interface.poll_firmware_download(cx)).await;
                        cp.firmware_download_response(res).await;
                    }
                    FirmwareState::Installing => {
                        let res =
                            poll_fn(|cx| cp.interface.interface.poll_firmware_install(cx)).await;
                        cp.firmware_install_response(res).await;
                    }
                    _ => {}
                }
                match &cp.diagnostics_state {
                    DiagnosticsState::Uploading(_) => {
                        let res =
                            poll_fn(|cx| cp.interface.interface.poll_diagnostics_upload(cx)).await;
                        cp.handle_diagnostics_response(res).await;
                    }
                    _ => {}
                }
                soft_reset = true;
                break;
            }
        }
        let ChargePoint { interface, .. } = cp;
        (interface, soft_reset)
    }

    pub async fn run(interface: I, mut configs: ChargePointConfig) {
        let mut interface = ChargePointBackend::new(interface);
        interface
            .init(configs.default_ocpp_configs.clone(), configs.clear_db)
            .await;
        loop {
            let (i, soft_reset) = Self::run_once(interface, configs.clone()).await;
            if !soft_reset {
                break;
            }
            interface = i;
            configs.seed = configs.seed.saturating_add(1);
        }
    }
}
