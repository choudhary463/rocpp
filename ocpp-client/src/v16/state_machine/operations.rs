use alloc::string::String;
use rocpp_core::v16::types::{Reason, RegistrationStatus, ResetType};

use crate::v16::{
    cp::ChargePoint,
    interfaces::{ChargePointInterface, TimerId},
};

use super::{call::OutgoingCallState, connector::ConnectorState};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) fn call_permission(&self) -> bool {
        self.ws_connected && self.registration_status == RegistrationStatus::Accepted
    }
    pub(crate) async fn handle_id_tag_authorized(
        &mut self,
        connector_id: usize,
        id_tag: String,
        parent_id_tag: Option<String>,
    ) {
        match &self.connector_state[connector_id] {
            ConnectorState::Idle => {
                if !self.firmware_state.ongoing_firmware_update() && self.pending_reset.is_none() {
                    self.add_timeout(
                        TimerId::Authorize(connector_id),
                        self.configs.connection_time_out.value,
                    )
                    .await;
                    self.change_connector_state(
                        connector_id,
                        ConnectorState::authorized(id_tag, parent_id_tag, None),
                    )
                    .await;
                }
            }
            ConnectorState::Plugged => {
                if !self.firmware_state.ongoing_firmware_update() && self.pending_reset.is_none() {
                    self.start_transaction(connector_id, id_tag, parent_id_tag, None)
                        .await;
                }
            }
            ConnectorState::Authorized {
                id_tag: auth_id_tag,
                parent_id_tag: auth_parent_id_tag,
                ..
            } => {
                if *auth_id_tag == id_tag
                    || (auth_parent_id_tag.is_some() && *auth_parent_id_tag == parent_id_tag)
                {
                    // same id tag presented, extend or deauthorize?, TC_061_CS for the future reference
                }
            }
            ConnectorState::Transaction {
                id_tag: transaction_id_tag,
                parent_id_tag: transaction_parent_id_tag,
                ..
            } => {
                if *transaction_id_tag == id_tag
                    || (transaction_parent_id_tag.is_some()
                        && *transaction_parent_id_tag == parent_id_tag)
                {
                    self.stop_transaction(connector_id, Some(id_tag), Some(Reason::Local))
                        .await;
                }
            }
            ConnectorState::Finishing => {
                // remove cable first
            }
            ConnectorState::Reserved {
                id_tag: reservation_id_tag,
                parent_id_tag: reservation_parent_id_tag,
                reservation_id,
                is_plugged,
            } => {
                if (id_tag == *reservation_id_tag
                    || (parent_id_tag.is_some() && *reservation_parent_id_tag == parent_id_tag))
                    && !self.firmware_state.ongoing_firmware_update()
                    && self.pending_reset.is_none()
                {
                    let reservation_id = *reservation_id;
                    let is_plugged = *is_plugged;
                    self.remove_reservation(connector_id, reservation_id).await;
                    if is_plugged {
                        self.start_transaction(
                            connector_id,
                            id_tag,
                            parent_id_tag,
                            Some(reservation_id),
                        )
                        .await;
                    } else {
                        self.change_connector_state(
                            connector_id,
                            ConnectorState::authorized(id_tag, parent_id_tag, Some(reservation_id)),
                        )
                        .await;
                    }
                    self.remove_timeout(TimerId::Reservation(connector_id))
                        .await;
                }
            }
            ConnectorState::Unavailable(_) => {
                // ignore
            }
            ConnectorState::Faulty => {
                // ignore
            }
        };
    }
    pub async fn reset(&mut self, kind: ResetType, reason: Option<Reason>) {
        self.pending_reset = Some(kind.clone());
        let reason = reason.or(match kind {
            ResetType::Hard => Some(Reason::HardReset),
            ResetType::Soft => Some(Reason::SoftReset),
        });
        for connector_id in 0..self.configs.number_of_connectors.value {
            if self.connector_state[connector_id].in_transaction() {
                self.stop_transaction(connector_id, None, reason.clone())
                    .await;
            }
        }
        match kind {
            ResetType::Soft => {
                if self.outgoing_call_state == OutgoingCallState::Idle {
                    self.soft_reset();
                }
            }
            ResetType::Hard => {
                self.interface.interface.hard_reset().await;
            }
        }
    }
}
