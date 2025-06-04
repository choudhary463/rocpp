use alloc::string::String;
use rocpp_core::v16::{
    messages::set_charging_profile::SetChargingProfileRequest, protocol_error::ProtocolError,
};

use crate::v16::{cp::ChargePoint, interfaces::ChargePointInterface};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn set_charging_profile_ocpp(
        &mut self,
        unique_id: String,
        _req: SetChargingProfileRequest,
    ) {
        self.send_error(unique_id, ProtocolError::NotImplemented)
            .await;
    }
}
