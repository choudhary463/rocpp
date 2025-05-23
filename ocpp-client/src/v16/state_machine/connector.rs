use alloc::string::String;
use ocpp_core::v16::{
    messages::status_notification::StatusNotificationRequest,
    types::{ChargePointErrorCode, ChargePointStatus},
};

use crate::v16::{
    interface::{Database, Secc, SeccState, TimerId},
    cp::ChargePointCore
};

use super::call::CallAction;

#[derive(Clone, Debug)]
pub(crate) enum ConnectorState {
    Idle,
    Plugged,
    Authorized {
        id_tag: String,
        parent_id_tag: Option<String>,
        reservation_id: Option<i32>,
    },
    Transaction {
        local_transaction_id: u32,
        id_tag: String,
        parent_id_tag: Option<String>,
        is_evse_suspended: bool,
        secc_state: SeccState,
    },
    Finishing,
    Reserved {
        reservation_id: i32,
        id_tag: String,
        parent_id_tag: Option<String>,
        is_plugged: bool,
    },
    Unavailable(SeccState),
    Faulty,
}

impl ConnectorState {
    pub fn idle() -> Self {
        Self::Idle
    }
    pub fn plugged() -> Self {
        Self::Plugged
    }
    pub fn authorized(
        id_tag: String,
        parent_id_tag: Option<String>,
        reservation_id: Option<i32>,
    ) -> Self {
        Self::Authorized {
            id_tag,
            parent_id_tag,
            reservation_id,
        }
    }
    pub fn transaction(
        local_transaction_id: u32,
        id_tag: String,
        parent_id_tag: Option<String>,
        is_evse_suspended: bool,
        secc_state: SeccState,
    ) -> Self {
        Self::Transaction {
            local_transaction_id,
            id_tag,
            parent_id_tag,
            is_evse_suspended,
            secc_state,
        }
    }
    pub fn finishing() -> Self {
        Self::Finishing
    }
    pub fn reserved(
        reservation_id: i32,
        id_tag: String,
        parent_id_tag: Option<String>,
        is_plugged: bool,
    ) -> Self {
        Self::Reserved {
            reservation_id,
            id_tag,
            parent_id_tag,
            is_plugged,
        }
    }
    pub fn unavailabe(secc_state: SeccState) -> Self {
        Self::Unavailable(secc_state)
    }
    pub fn faulty() -> Self {
        Self::Faulty
    }
    pub fn in_transaction(&self) -> bool {
        matches!(self, ConnectorState::Transaction { .. })
    }
    pub fn get_connector_state(&self, new_firmware: bool) -> ChargePointStatus {
        let mut in_transaction = false;
        let mut res = match &self {
            ConnectorState::Idle => ChargePointStatus::Available,
            ConnectorState::Authorized { .. } => ChargePointStatus::Preparing,
            ConnectorState::Plugged => ChargePointStatus::Preparing,
            ConnectorState::Transaction {
                secc_state,
                is_evse_suspended,
                ..
            } => {
                in_transaction = true;
                match secc_state {
                    SeccState::Faulty => ChargePointStatus::Faulted,
                    SeccState::Plugged => {
                        if *is_evse_suspended {
                            ChargePointStatus::SuspendedEVSE
                        } else {
                            ChargePointStatus::Charging
                        }
                    }
                    SeccState::Unplugged => ChargePointStatus::SuspendedEV,
                }
            }
            ConnectorState::Finishing => ChargePointStatus::Finishing,
            ConnectorState::Reserved { .. } => ChargePointStatus::Reserved,
            ConnectorState::Unavailable(secc_state) => match secc_state {
                SeccState::Faulty => ChargePointStatus::Faulted,
                _ => ChargePointStatus::Unavailable,
            },
            ConnectorState::Faulty => ChargePointStatus::Faulted,
        };
        if !in_transaction && new_firmware {
            res = ChargePointStatus::Unavailable;
        }
        res
    }
}

#[derive(Clone, Debug)]
pub enum StatusNotificationState {
    Offline(Option<ChargePointStatus>),
    Idle,
    Stabilizing(ChargePointStatus),
}

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub(crate) fn send_status_notification(&mut self, connector_id: usize) {
        self.enqueue_call(
            CallAction::StatusNotification,
            self.connector_status_notification[connector_id].clone(),
        );
        self.connector_status_notification_state[connector_id] = StatusNotificationState::Idle;
    }
    pub(crate) fn on_status_notification_online(&mut self) {
        for connector_id in 0..self.configs.number_of_connectors.value {
            match &self.connector_status_notification_state[connector_id] {
                StatusNotificationState::Offline(last_sent) => {
                    if last_sent
                        .as_ref()
                        .map(|f| *f != self.connector_status_notification[connector_id].status)
                        .unwrap_or(true)
                    {
                        self.secc.update_status(
                            connector_id,
                            self.connector_status_notification[connector_id]
                                .status
                                .clone(),
                        );
                        self.send_status_notification(connector_id);
                    } else {
                        self.connector_status_notification_state[connector_id] =
                            StatusNotificationState::Idle;
                    }
                }
                _ => {
                    unreachable!();
                }
            }
        }
    }
    pub(crate) fn on_status_notification_offline(&mut self) {
        for connector_id in 0..self.configs.number_of_connectors.value {
            match &self.connector_status_notification_state[connector_id] {
                StatusNotificationState::Idle => {
                    self.connector_status_notification_state[connector_id] =
                        StatusNotificationState::Offline(Some(
                            self.connector_status_notification[connector_id]
                                .status
                                .clone(),
                        ));
                }
                StatusNotificationState::Stabilizing(last_sent) => {
                    self.connector_status_notification_state[connector_id] =
                        StatusNotificationState::Offline(Some(last_sent.clone()))
                }
                _ => {
                    unreachable!();
                }
            }
        }
    }
    pub(crate) fn sync_connector_states(
        &mut self,
        connector_id: usize,
        error_code: Option<ChargePointErrorCode>,
        info: Option<String>,
    ) {
        let new_status_notification_state = self.connector_state[connector_id]
            .get_connector_state(self.firmware_state.ongoing_firmware_update());
        if new_status_notification_state != self.connector_status_notification[connector_id].status
        {
            self.secc
                .update_status(connector_id, new_status_notification_state.clone());
            self.update_status_notification_state(
                connector_id,
                new_status_notification_state,
                error_code.unwrap_or(ChargePointErrorCode::NoError),
                info,
            );
        }
    }
    pub(crate) fn change_connector_state_with_error_code(
        &mut self,
        connector_id: usize,
        state: ConnectorState,
        error_code: Option<ChargePointErrorCode>,
        info: Option<String>,
    ) {
        self.connector_state[connector_id] = state;
        self.sync_connector_states(connector_id, error_code, info);
    }
    pub(crate) fn change_connector_state(&mut self, connector_id: usize, state: ConnectorState) {
        self.change_connector_state_with_error_code(connector_id, state, None, None);
    }
    pub(crate) fn trigger_status_notification(&mut self, connector_id: usize) {
        for connector_id in if connector_id == 0 {
            0..self.configs.number_of_connectors.value
        } else {
            (connector_id - 1)..(connector_id)
        } {
            self.send_status_notification(connector_id);
        }
    }
    fn stabilize(&mut self, connector_id: usize, last_sent_status: ChargePointStatus) {
        self.add_timeout(
            TimerId::StatusNotification(connector_id),
            self.configs.minimum_status_duration.value,
        );
        self.connector_status_notification_state[connector_id] =
            StatusNotificationState::Stabilizing(last_sent_status);
    }

    fn update_status_notification_state(
        &mut self,
        connector_id: usize,
        status: ChargePointStatus,
        error_code: ChargePointErrorCode,
        info: Option<String>,
    ) {
        let previous_status = self.connector_status_notification[connector_id]
            .status
            .clone();
        self.connector_status_notification[connector_id] = StatusNotificationRequest {
            connector_id: (connector_id + 1),
            error_code,
            info,
            status: status.clone(),
            timestamp: self.get_time(),
            vendor_id: None,
            vendor_error_code: None,
        };
        match &self.connector_status_notification_state[connector_id] {
            StatusNotificationState::Idle => {
                if previous_status != status {
                    self.stabilize(connector_id, previous_status);
                }
            }
            StatusNotificationState::Stabilizing(last_sent) => {
                if *last_sent != status {
                    self.stabilize(connector_id, last_sent.clone());
                } else {
                    self.remove_timeout(TimerId::StatusNotification(connector_id));
                    self.connector_status_notification_state[connector_id] =
                        StatusNotificationState::Idle;
                }
            }
            _ => {}
        }
    }
}
