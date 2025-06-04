use alloc::string::String;
use rocpp_core::v16::types::{ChargePointErrorCode, Reason};

use crate::v16::{
    cp::ChargePoint,
    interfaces::{ChargePointInterface, SeccState, TimerId},
    state_machine::{auth::AuthorizeStatus, connector::ConnectorState},
};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub async fn secc_id_tag(&mut self, connector_id: usize, id_tag: String) {
        match self.evaluate_id_tag_auth(id_tag, connector_id).await {
            AuthorizeStatus::NotAuthorized => {}
            AuthorizeStatus::Authorized {
                connector_id,
                id_tag,
                parent_id_tag,
            } => {
                self.handle_id_tag_authorized(connector_id, id_tag, parent_id_tag)
                    .await;
            }
            AuthorizeStatus::SendAuthorize {
                connector_id,
                id_tag,
            } => {
                self.send_authorize_request(connector_id, id_tag).await;
            }
        };
    }

    pub async fn secc_change_state(
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
                        )
                        .await;
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
                        )
                        .await;
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
                        )
                        .await;
                    }
                    SeccState::Faulty => {
                        self.change_connector_state_with_error_code(
                            connector_id,
                            ConnectorState::faulty(),
                            error_code,
                            info,
                        )
                        .await;
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
                        self.remove_timeout(TimerId::Authorize(connector_id)).await;
                        self.start_transaction(connector_id, id_tag, parent_id_tag, reservation_id)
                            .await;
                    }
                    SeccState::Unplugged => {
                        // Authorized + Unplugged ??
                    }
                    SeccState::Faulty => {
                        self.remove_timeout(TimerId::Authorize(connector_id)).await;
                        self.change_connector_state_with_error_code(
                            connector_id,
                            ConnectorState::faulty(),
                            error_code,
                            info,
                        )
                        .await;
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
                if *secc_state != state
                    && state == SeccState::Unplugged
                    && self.configs.stop_transaction_on_evside_disconnect.value
                {
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
                    )
                    .await;
                    self.stop_transaction(connector_id, None, Some(Reason::EVDisconnected))
                        .await;
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
                )
                .await;
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
                        )
                        .await;
                    }
                    SeccState::Faulty => {
                        self.change_connector_state_with_error_code(
                            connector_id,
                            ConnectorState::faulty(),
                            error_code,
                            info,
                        )
                        .await;
                    }
                }
            }
            ConnectorState::Reserved {
                reservation_id,
                id_tag,
                parent_id_tag,
                ..
            } => {
                let is_plugged = match state {
                    SeccState::Plugged => true,
                    SeccState::Unplugged => false,
                    SeccState::Faulty => {
                        self.remove_reservation(connector_id, *reservation_id).await;
                        self.change_connector_state_with_error_code(
                            connector_id,
                            ConnectorState::faulty(),
                            error_code,
                            info,
                        )
                        .await;
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
                )
                .await;
            }
            ConnectorState::Unavailable(_) => {
                self.change_connector_state_with_error_code(
                    connector_id,
                    ConnectorState::Unavailable(state),
                    error_code,
                    info,
                )
                .await;
            }
            ConnectorState::Faulty => match state {
                SeccState::Plugged => {
                    self.change_connector_state_with_error_code(
                        connector_id,
                        ConnectorState::plugged(),
                        error_code,
                        info,
                    )
                    .await;
                }
                SeccState::Unplugged => {
                    self.change_connector_state_with_error_code(
                        connector_id,
                        ConnectorState::idle(),
                        error_code,
                        info,
                    )
                    .await;
                }
                SeccState::Faulty => {
                    self.change_connector_state_with_error_code(
                        connector_id,
                        ConnectorState::faulty(),
                        error_code,
                        info,
                    )
                    .await;
                }
            },
        }
    }
}
