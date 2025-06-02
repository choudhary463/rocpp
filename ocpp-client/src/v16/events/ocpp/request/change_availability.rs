use alloc::{string::String, vec::Vec};
use ocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::change_availability::{ChangeAvailabilityRequest, ChangeAvailabilityResponse},
        types::{AvailabilityStatus, AvailabilityType},
    },
};

use crate::v16::{
    cp::core::ChargePointCore, drivers::{database::Database, hardware_interface::HardwareInterface, peripheral_input::SeccState, timers::TimerId}, state_machine::connector::ConnectorState
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn change_availability_ocpp(&mut self, unique_id: String, req: ChangeAvailabilityRequest) {
        let mut changes = Vec::new();
        let mut pending = false;
        if req.connector_id > self.configs.number_of_connectors.value {
            let payload = ChangeAvailabilityResponse {
                status: AvailabilityStatus::Rejected,
            };
            let res = CallResult::new(unique_id, payload);
            self.send_ws_msg(res.encode());
            return;
        }
        for connector_id in if req.connector_id == 0 {
            0..self.configs.number_of_connectors.value
        } else {
            (req.connector_id - 1)..(req.connector_id)
        } {
            match &req.kind {
                AvailabilityType::Operative => {
                    if let ConnectorState::Unavailable(secc_state) =
                        &self.connector_state[connector_id]
                    {
                        match secc_state {
                            SeccState::Plugged => {
                                changes.push((connector_id, ConnectorState::plugged()))
                            }
                            SeccState::Unplugged => {
                                changes.push((connector_id, ConnectorState::idle()))
                            }
                            SeccState::Faulty => {
                                changes.push((connector_id, ConnectorState::faulty()))
                            }
                        }
                    }
                }
                AvailabilityType::Inoperative => match &self.connector_state[connector_id] {
                    ConnectorState::Idle => {
                        changes.push((
                            connector_id,
                            ConnectorState::unavailabe(SeccState::Unplugged),
                        ));
                    }
                    ConnectorState::Plugged => {
                        changes
                            .push((connector_id, ConnectorState::unavailabe(SeccState::Plugged)));
                    }
                    ConnectorState::Authorized { .. } => {
                        self.remove_timeout(TimerId::Authorize(connector_id));
                        changes.push((
                            connector_id,
                            ConnectorState::unavailabe(SeccState::Unplugged),
                        ));
                    }
                    ConnectorState::Transaction { .. } => {
                        pending = true;
                        self.pending_inoperative_changes[connector_id] = true;
                    }
                    ConnectorState::Finishing => {
                        changes
                            .push((connector_id, ConnectorState::unavailabe(SeccState::Plugged)));
                    }
                    ConnectorState::Reserved {
                        is_plugged,
                        reservation_id,
                        ..
                    } => {
                        let secc_state = if *is_plugged {
                            SeccState::Plugged
                        } else {
                            SeccState::Unplugged
                        };
                        self.remove_reservation(connector_id, *reservation_id);
                        changes.push((connector_id, ConnectorState::unavailabe(secc_state)));
                    }
                    ConnectorState::Faulty => {
                        changes.push((connector_id, ConnectorState::unavailabe(SeccState::Faulty)));
                    }
                    _ => {}
                },
            }
            self.db
                .db_change_operative_state(connector_id, req.kind.clone());
        }
        let status = if pending {
            AvailabilityStatus::Scheduled
        } else {
            AvailabilityStatus::Accepted
        };
        let payload = ChangeAvailabilityResponse { status };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode());

        for (connector_id, state) in changes {
            self.change_connector_state(connector_id, state);
        }
    }
}
