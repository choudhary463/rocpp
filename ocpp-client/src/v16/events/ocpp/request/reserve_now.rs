use alloc::string::String;
use ocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::reserve_now::{ReserveNowRequest, ReserveNowResponse},
        types::ReservationStatus,
    },
};

use crate::v16::{
    cp::core::ChargePointCore, drivers::{database::Database, hardware_interface::HardwareInterface, peripheral_input::SeccState, timers::TimerId}, state_machine::connector::ConnectorState
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn reserve_now_ocpp(&mut self, unique_id: String, req: ReserveNowRequest) {
        let mut send_status = None;
        let connector_id = req.connector_id;
        let already_reserved = self.get_connector_with_reservation(req.reservation_id);
        let mut status = ReservationStatus::Rejected;
        let current = self.get_time().unwrap();
        let diff = req.expiry_date - current;
        if connector_id >= 1
            && connector_id <= self.configs.number_of_connectors.value
            && already_reserved
                .map(|f| f == connector_id - 1)
                .unwrap_or(true)
            && diff.num_seconds() > 0
        {
            let connector_id = req.connector_id - 1;
            match &mut self.connector_state[connector_id] {
                ConnectorState::Idle => {
                    self.db.db_add_reservation(req.clone());
                    self.add_timeout(
                        TimerId::Reservation(connector_id),
                        diff.num_seconds() as u64,
                    );
                    send_status = Some((
                        connector_id,
                        ConnectorState::reserved(
                            req.reservation_id,
                            req.id_tag,
                            req.parent_id_tag,
                            false,
                        ),
                    ));
                    status = ReservationStatus::Accepted;
                }
                ConnectorState::Plugged => {
                    status = ReservationStatus::Occupied;
                }
                ConnectorState::Authorized { .. } => {
                    status = ReservationStatus::Occupied;
                }
                ConnectorState::Transaction { .. } => {
                    status = ReservationStatus::Occupied;
                }
                ConnectorState::Finishing => {
                    status = ReservationStatus::Occupied;
                }
                ConnectorState::Reserved {
                    id_tag: reservation_id_tag,
                    parent_id_tag: reservation_parent_id_tag,
                    reservation_id,
                    ..
                } => {
                    if req.reservation_id == *reservation_id {
                        self.db.db_add_reservation(req.clone());
                        *reservation_id_tag = req.id_tag;
                        *reservation_parent_id_tag = req.parent_id_tag;
                        status = ReservationStatus::Accepted;
                        self.add_timeout(
                            TimerId::Reservation(connector_id),
                            diff.num_seconds() as u64,
                        );
                    } else {
                        status = ReservationStatus::Occupied;
                    }
                }
                ConnectorState::Unavailable(secc_state) => match secc_state {
                    SeccState::Faulty => status = ReservationStatus::Faulted,
                    _ => status = ReservationStatus::Unavailable,
                },
                ConnectorState::Faulty => {
                    status = ReservationStatus::Faulted;
                }
            }
        }
        let payload = ReserveNowResponse { status };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode());
        if let Some((connector_id, state)) = send_status {
            self.change_connector_state(connector_id, state);
        }
    }
}
