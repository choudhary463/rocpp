use crate::v16::{
    interface::{Database, Secc},
    services::timeout::TimerId,
};

use super::core::ChargePointCore;

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn remove_reservation(&mut self, connector_id: usize, reservation_id: i32) {
        self.db.db_remove_reservation(reservation_id);
        self.remove_timeout(TimerId::Reservation(connector_id));
    }
}
