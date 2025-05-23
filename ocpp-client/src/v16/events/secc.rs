use alloc::string::String;
use ocpp_core::v16::types::{ChargePointErrorCode, Reason};

use crate::v16::{
    cp::ChargePointCore, interface::{Database, Secc, SeccState, TimerId}, state_machine::{auth::AuthorizeStatus, connector::ConnectorState}
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn secc_id_tag_helper(&mut self, connector_id: usize, id_tag: String) {
        match self.evaluate_id_tag_auth(id_tag, connector_id) {
            AuthorizeStatus::NotAuthorized => {}
            AuthorizeStatus::Authorized {
                connector_id,
                id_tag,
                parent_id_tag,
            } => {
                self.handle_id_tag_authorized(connector_id, id_tag, parent_id_tag);
            }
            AuthorizeStatus::SendAuthorize {
                connector_id,
                id_tag,
            } => {
                self.send_authorize_request(connector_id, id_tag);
            }
        };
    }

    pub fn secc_change_state_helper(
        &mut self,
        connector_id: usize,
        state: SeccState,
        error_code: Option<ChargePointErrorCode>,
        info: Option<String>,
    ) {
        match &self.connector_state[connector_id] {
            ConnectorState::Idle => {
                match state {
                    SeccState::Plugged => {
                        self.change_connector_state_with_error_code(
                            connector_id,
                            ConnectorState::plugged(),
                            error_code,
                            info,
                        );
                    }
                    SeccState::Unplugged => {
                        // idle -> Unplugged ??
                    }
                    SeccState::Faulty => {
                        self.change_connector_state_with_error_code(
                            connector_id,
                            ConnectorState::faulty(),
                            error_code,
                            info,
                        );
                    }
                }
            }
            ConnectorState::Plugged => {
                match state {
                    SeccState::Plugged => {
                        // Plugged + Plugged ??
                    }
                    SeccState::Unplugged => {
                        self.change_connector_state_with_error_code(
                            connector_id,
                            ConnectorState::idle(),
                            error_code,
                            info,
                        );
                    }
                    SeccState::Faulty => {
                        self.change_connector_state_with_error_code(
                            connector_id,
                            ConnectorState::faulty(),
                            error_code,
                            info,
                        );
                    }
                }
            }
            ConnectorState::Authorized {
                id_tag,
                parent_id_tag,
                reservation_id,
            } => {
                match state {
                    SeccState::Plugged => {
                        let id_tag = id_tag.clone();
                        let parent_id_tag = parent_id_tag.clone();
                        let reservation_id = *reservation_id;
                        self.remove_timeout(TimerId::Authorize(connector_id));
                        self.start_transaction(connector_id, id_tag, parent_id_tag, reservation_id);
                    }
                    SeccState::Unplugged => {
                        // Authorized + Unplugged ??
                    }
                    SeccState::Faulty => {
                        self.remove_timeout(TimerId::Authorize(connector_id));
                        self.change_connector_state_with_error_code(
                            connector_id,
                            ConnectorState::faulty(),
                            error_code,
                            info,
                        );
                    }
                }
            }
            ConnectorState::Transaction {
                local_transaction_id,
                id_tag,
                parent_id_tag,
                is_evse_suspended,
                secc_state,
            } => {
                if *secc_state != state && state == SeccState::Unplugged && self.configs.stop_transaction_on_evside_disconnect.value {
                    self.change_connector_state_with_error_code(
                        connector_id,
                        ConnectorState::transaction(
                            *local_transaction_id,
                            id_tag.clone(),
                            parent_id_tag.clone(),
                            *is_evse_suspended,
                            state,
                        ),
                        error_code,
                        info,
                    );
                    self.stop_transaction(
                        connector_id,
                        None,
                        Some(Reason::EVDisconnected),
                    );
                    return;
                }
                self.change_connector_state_with_error_code(
                    connector_id,
                    ConnectorState::transaction(
                        *local_transaction_id,
                        id_tag.clone(),
                        parent_id_tag.clone(),
                        *is_evse_suspended,
                        state,
                    ),
                    error_code,
                    info,
                );
            }
            ConnectorState::Finishing => {
                match state {
                    SeccState::Plugged => {
                        // Finishing + Plugged
                    }
                    SeccState::Unplugged => {
                        self.change_connector_state_with_error_code(
                            connector_id,
                            ConnectorState::idle(),
                            error_code,
                            info,
                        );
                    }
                    SeccState::Faulty => {
                        self.change_connector_state_with_error_code(
                            connector_id,
                            ConnectorState::faulty(),
                            error_code,
                            info,
                        );
                    }
                }
            }
            ConnectorState::Reserved {
                reservation_id,
                id_tag,
                parent_id_tag,
                ..
            } => {
                let is_plugged= match state {
                    SeccState::Plugged => {
                        true
                    }
                    SeccState::Unplugged => {
                        false
                    }
                    SeccState::Faulty => {
                        self.remove_reservation(connector_id, *reservation_id);
                        self.change_connector_state_with_error_code(
                            connector_id,
                            ConnectorState::faulty(),
                            error_code,
                            info,
                        );
                        return;
                    }
                };
                self.change_connector_state_with_error_code(
                    connector_id,
                    ConnectorState::reserved(
                        *reservation_id,
                        id_tag.clone(),
                        parent_id_tag.clone(),
                        is_plugged,
                    ),
                    error_code,
                    info,
                );
            }
            ConnectorState::Unavailable(_) => {
                self.change_connector_state_with_error_code(
                    connector_id,
                    ConnectorState::Unavailable(state),
                    error_code,
                    info,
                );
            }
            ConnectorState::Faulty => match state {
                SeccState::Plugged => {
                    self.change_connector_state_with_error_code(
                        connector_id,
                        ConnectorState::plugged(),
                        error_code,
                        info,
                    );
                }
                SeccState::Unplugged => {
                    self.change_connector_state_with_error_code(
                        connector_id,
                        ConnectorState::idle(),
                        error_code,
                        info,
                    );
                }
                SeccState::Faulty => {
                    self.change_connector_state_with_error_code(
                        connector_id,
                        ConnectorState::faulty(),
                        error_code,
                        info,
                    );
                }
            },
        }
    }
}
