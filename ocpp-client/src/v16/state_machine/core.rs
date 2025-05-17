use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
};

use chrono::{DateTime, Utc};
use ocpp_core::{
    format::frame::Call,
    v16::{
        messages::{
            boot_notification::BootNotificationRequest,
            status_notification::StatusNotificationRequest,
        },
        protocol_error::ProtocolError,
        types::{IdTagInfo, RegistrationStatus, ResetType},
    },
};

use crate::v16::{
    interface::{Database, Secc},
    services::{database::DatabaseService, secc::SeccService},
};

use super::{
    actions::CoreActions,
    auth::CachedEntry,
    boot::BootState,
    call::{CallAction, OutgoingCallState},
    config::OcppConfigs,
    connector::{ConnectorState, StatusNotificationState},
    diagnostics::DiagnosticsState,
    firmware::{FirmwareInstallStatus, FirmwareState},
    heartbeat::HeartbeatState,
    meter::MeterState,
    transaction::{MeterValueLocal, TransactionEvent, TransactionEventState},
};

pub type OcppError = ocpp_core::format::error::OcppError<ProtocolError>;

pub(crate) struct ChargePointCore<D: Database, S: Secc> {
    pub db: DatabaseService<D>,
    pub secc: SeccService<S>,
    pub cms_url: String,
    pub boot_info: BootNotificationRequest,
    pub ws_connected: bool,
    pub queued_actions: VecDeque<CoreActions>,
    pub call_timeout: u64,
    pub outgoing_call_state: OutgoingCallState,
    pub pending_calls: VecDeque<(Call, CallAction)>,
    pub boot_state: BootState,
    pub registration_status: RegistrationStatus,
    pub heartbeat_state: HeartbeatState,
    pub base_time: Option<(DateTime<Utc>, Instant)>,
    pub authorization_cache: HashMap<String, CachedEntry>,
    pub cache_usage_order: VecDeque<String>,
    pub max_cache_len: usize,
    pub local_list_version: i32,
    pub local_list_entries: HashMap<String, IdTagInfo>,
    pub pending_auth_requests: VecDeque<(usize, String)>,
    pub connector_state: Vec<ConnectorState>,
    pub connector_status_notification: Vec<StatusNotificationRequest>,
    pub connector_status_notification_state: Vec<StatusNotificationState>,
    pub pending_inoperative_changes: Vec<bool>,
    pub sampled_meter_state: Vec<MeterState>,
    pub aligned_meter_state: MeterState,
    pub local_transaction_id: u32,
    pub active_local_transactions: Vec<Option<(u32, Option<i32>)>>,
    pub transaction_head: u64,
    pub transaction_tail: u64,
    pub transaction_queue: VecDeque<TransactionEvent>,
    pub transaction_map: HashMap<u32, i32>,
    pub transaction_connector_map: HashMap<u32, usize>,
    pub transaction_stop_meter_map: HashMap<u32, Vec<MeterValueLocal>>,
    pub transaction_event_state: TransactionEventState,
    pub transaction_event_retries: u64,
    pub diagnostics_state: DiagnosticsState,
    pub firmware_state: FirmwareState,
    pub last_firmware_state: FirmwareInstallStatus,
    pub pending_reset: Option<ResetType>,
    pub configs: OcppConfigs,
}

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn new(
        mut db: DatabaseService<D>,
        secc: SeccService<S>,
        cms_url: String,
        call_timeout: u64,
        max_cache_len: usize,
        boot_info: BootNotificationRequest,
    ) -> Self {
        let db_configs = db.get_all_config();

        let configs = OcppConfigs::build(db_configs);

        let num_connectors = configs.number_of_connectors.value;

        let (authorization_cache, cache_usage_order) = db.get_cache_data();
        let (local_list_version, local_list_entries) = db.get_local_list();

        let (connector_state, connector_status_notification) =
            db.get_connector_state(num_connectors);

        let (
            local_transaction_id,
            transaction_tail,
            transaction_head,
            transaction_map,
            transaction_connector_map,
            transaction_stop_meter_map,
            transaction_queue,
        ) = db.get_transaction_data();

        let last_firmware_state = db.get_firmware_state();

        Self {
            db,
            secc,
            cms_url,
            boot_info,
            ws_connected: false,
            queued_actions: VecDeque::new(),
            call_timeout,
            outgoing_call_state: OutgoingCallState::Idle,
            pending_calls: VecDeque::new(),
            boot_state: BootState::Idle,
            registration_status: RegistrationStatus::Rejected,
            heartbeat_state: HeartbeatState::Idle,
            base_time: None,
            authorization_cache,
            cache_usage_order,
            max_cache_len,
            local_list_version,
            local_list_entries,
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
            transaction_queue,
            transaction_map,
            transaction_connector_map,
            transaction_stop_meter_map,
            transaction_event_state: TransactionEventState::Idle,
            transaction_event_retries: 0,
            diagnostics_state: DiagnosticsState::Idle,
            firmware_state: FirmwareState::Idle,
            last_firmware_state,
            pending_reset: None,
            configs,
        }
    }
}
