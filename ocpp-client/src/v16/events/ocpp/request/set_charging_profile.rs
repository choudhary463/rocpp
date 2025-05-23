use alloc::string::String;
use ocpp_core::v16::{
    messages::set_charging_profile::SetChargingProfileRequest, protocol_error::ProtocolError,
};

use crate::v16::{
    interface::{Database, Secc},
    cp::ChargePointCore,
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub(crate) fn set_charging_profile_ocpp(
        &mut self,
        unique_id: String,
        _req: SetChargingProfileRequest,
    ) {
        self.send_error(unique_id, ProtocolError::NotImplemented);
    }
}
