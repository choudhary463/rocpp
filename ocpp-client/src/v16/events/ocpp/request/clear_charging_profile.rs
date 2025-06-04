use alloc::string::String;
use rocpp_core::v16::{
    messages::clear_charging_profile::ClearChargingProfileRequest, protocol_error::ProtocolError,
};

use crate::v16::{cp::ChargePoint, interfaces::ChargePointInterface};


impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn clear_charging_profile_ocpp(
        &mut self,
        unique_id: String,
        _req: ClearChargingProfileRequest,
    ) {
        self.send_error(unique_id, ProtocolError::NotImplemented).await;
    }
}
