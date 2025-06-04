use crate::v16::{
    cp::ChargePoint,
    interfaces::{ChargePointInterface, TimerId},
};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn remove_reservation(&mut self, connector_id: usize, reservation_id: i32) {
        self.interface.db_remove_reservation(reservation_id).await;
        self.remove_timeout(TimerId::Reservation(connector_id))
            .await;
    }
}
