use alloc::string::String;
use ocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::cancel_reservation::{CancelReservationRequest, CancelReservationResponse},
        types::CancelReservationStatus,
    },
};

use crate::v16::{
    cp::core::ChargePointCore, drivers::{database::Database, hardware_interface::HardwareInterface}, state_machine::connector::ConnectorState
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn get_connector_with_reservation(&self, reservation_id: i32) -> Option<usize> {
        for connector_id in 0..self.configs.number_of_connectors.value {
            if let ConnectorState::Reserved {
                reservation_id: res_id,
                ..
                } = &self.connector_state[connector_id] {
                if *res_id == reservation_id {
                    return Some(connector_id);
                }
            }
        }
        None
    }
    pub(crate) fn cancel_reservation_ocpp(&mut self, unique_id: String, req: CancelReservationRequest) {
        let mut new_status = None;
        let status =
            if let Some(connector_id) = self.get_connector_with_reservation(req.reservation_id) {
                match &self.connector_state[connector_id] {
                    ConnectorState::Reserved { is_plugged, .. } => {
                        if *is_plugged {
                            new_status = Some((connector_id, ConnectorState::plugged()));
                        } else {
                            new_status = Some((connector_id, ConnectorState::idle()));
                        }
                    }
                    _ => {
                        unreachable!();
                    }
                }
                self.remove_reservation(connector_id, req.reservation_id);
                CancelReservationStatus::Accepted
            } else {
                CancelReservationStatus::Rejected
            };
        let payload = CancelReservationResponse { status };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode());
        if let Some((connector_id, state)) = new_status {
            self.change_connector_state(connector_id, state);
        }
    }
}
