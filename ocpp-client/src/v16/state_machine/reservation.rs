use crate::v16::{
    cp::core::ChargePointCore, drivers::{database::Database, hardware_interface::HardwareInterface, timers::TimerId}
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn remove_reservation(&mut self, connector_id: usize, reservation_id: i32) {
        self.db.db_remove_reservation(reservation_id);
        self.remove_timeout(TimerId::Reservation(connector_id));
    }
}
