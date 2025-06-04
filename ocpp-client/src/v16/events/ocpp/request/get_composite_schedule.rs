use alloc::string::String;
use rocpp_core::v16::{
    messages::get_composite_schedule::GetCompositeScheduleRequest, protocol_error::ProtocolError,
};

use crate::v16::{cp::ChargePoint, interfaces::ChargePointInterface};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn get_composite_schedule_ocpp(
        &mut self,
        unique_id: String,
        _req: GetCompositeScheduleRequest,
    ) {
        self.send_error(unique_id, ProtocolError::NotImplemented)
            .await;
    }
}
