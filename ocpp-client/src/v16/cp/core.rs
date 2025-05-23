use alloc::{collections::{btree_map::BTreeMap, vec_deque::VecDeque}, string::String, vec, vec::Vec};
use chrono::{DateTime, Utc};
use ocpp_core::{format::frame::Call, v16::{messages::{boot_notification::BootNotificationRequest, status_notification::StatusNotificationRequest}, protocol_error::ProtocolError, types::{ChargePointErrorCode, IdTagInfo, RegistrationStatus, ResetType}}};
use rand::{rngs::SmallRng, SeedableRng};

use crate::v16::{services::{database::DatabaseService, secc::SeccService}, state_machine::{actions::CoreActions, auth::CachedEntry, boot::BootState, call::{CallAction, OutgoingCallState}, clock::Instant, config::OcppConfigs, connector::{ConnectorState, StatusNotificationState}, diagnostics::DiagnosticsState, firmware::{FirmwareInstallStatus, FirmwareState}, heartbeat::HeartbeatState, meter::MeterState, transaction::{MeterValueLocal, TransactionEvent, TransactionEventState}}, Database, DiagnosticsResponse, Secc, SeccState, TimerId};

use super::config::ChargePointConfig;

pub(crate) type OcppError = ocpp_core::format::error::OcppError<ProtocolError>;

pub struct ChargePointCore<D: Database, S: Secc> {
    pub(crate) db: DatabaseService<D>,
    pub(crate) secc: SeccService<S>,
    pub(crate) rng: SmallRng,
    pub(crate) cms_url: String,
    pub(crate) boot_info: BootNotificationRequest,
    pub(crate) ws_connected: bool,
    pub(crate) queued_actions: VecDeque<CoreActions>,
    pub(crate) call_timeout: u64,
    pub(crate) outgoing_call_state: OutgoingCallState,
    pub(crate) pending_calls: VecDeque<(Call, CallAction)>,
    pub(crate) boot_state: BootState,
    pub(crate) registration_status: RegistrationStatus,
    pub(crate) heartbeat_state: HeartbeatState,
    pub(crate) base_time: Option<(DateTime<Utc>, Instant)>,
    pub(crate) authorization_cache: BTreeMap<String, CachedEntry>,
    pub(crate) cache_usage_order: VecDeque<String>,
    pub(crate) max_cache_len: usize,
    pub(crate) local_list_version: i32,
    pub(crate) local_list_entries: BTreeMap<String, IdTagInfo>,
    pub(crate) pending_auth_requests: VecDeque<(usize, String)>,
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
    pub(crate) transaction_queue: VecDeque<TransactionEvent>,
    pub(crate) transaction_map: BTreeMap<u32, i32>,
    pub(crate) transaction_connector_map: BTreeMap<u32, usize>,
    pub(crate) transaction_stop_meter_map: BTreeMap<u32, Vec<MeterValueLocal>>,
    pub(crate) transaction_event_state: TransactionEventState,
    pub(crate) transaction_event_retries: u64,
    pub(crate) diagnostics_state: DiagnosticsState,
    pub(crate) firmware_state: FirmwareState,
    pub(crate) last_firmware_state: FirmwareInstallStatus,
    pub(crate) pending_reset: Option<ResetType>,
    pub(crate) configs: OcppConfigs,
}

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn new(
        mut db: DatabaseService<D>,
        secc: SeccService<S>,
        cp_configs: ChargePointConfig
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
            rng: SmallRng::seed_from_u64(cp_configs.seed),
            cms_url: cp_configs.cms_url,
            boot_info: cp_configs.boot_info,
            ws_connected: false,
            queued_actions: VecDeque::new(),
            call_timeout: cp_configs.call_timeout,
            outgoing_call_state: OutgoingCallState::Idle,
            pending_calls: VecDeque::new(),
            boot_state: BootState::Idle,
            registration_status: RegistrationStatus::Rejected,
            heartbeat_state: HeartbeatState::Idle,
            base_time: None,
            authorization_cache,
            cache_usage_order,
            max_cache_len: cp_configs.max_cache_len,
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

    pub fn init(&mut self) {
        self.init_helper();
    }
    pub fn secc_change_state(&mut self, connector_id: usize, state: SeccState, error_code: Option<ChargePointErrorCode>, info: Option<String>,) -> Vec<CoreActions> {
        self.secc_change_state_helper(connector_id, state, error_code, info);
        self.queued_actions.drain(..).collect()
    }
    pub fn secc_id_tag(&mut self, connector_id: usize, id_tag: String) -> Vec<CoreActions> {
        self.secc_id_tag_helper(connector_id, id_tag);
        self.queued_actions.drain(..).collect()
    }
    pub fn ws_connected(&mut self) -> Vec<CoreActions> {
        self.ws_connected_helper();
        self.queued_actions.drain(..).collect()
    }
    pub fn ws_disconnected(&mut self) -> Vec<CoreActions> {
        self.ws_disconnected_helper();
        self.queued_actions.drain(..).collect()
    }
    pub fn got_ws_msg(&mut self, msg: String) -> Vec<CoreActions> {
        self.got_ws_msg_helper(msg);
        self.queued_actions.drain(..).collect()
    }
    pub fn handle_timeout(&mut self, id: TimerId) -> Vec<CoreActions> {
        self.handle_timeout_helper(id);
        self.queued_actions.drain(..).collect()
    }
    pub fn firmware_download_response(&mut self, res: Option<Vec<u8>>) -> Vec<CoreActions> {
        self.firmware_download_response_helper(res);
        self.queued_actions.drain(..).collect()
    }
    pub fn firmware_install_response(&mut self, res: bool) -> Vec<CoreActions> {
        self.firmware_install_response_helper(res);
        self.queued_actions.drain(..).collect()
    }
    pub fn handle_diagnostics_response(&mut self, upload_status: DiagnosticsResponse) -> Vec<CoreActions> {
        self.handle_diagnostics_response_helper(upload_status);
        self.queued_actions.drain(..).collect()
    }
}
