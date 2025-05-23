use alloc::string::String;
use ocpp_core::v16::{
    messages::clear_charging_profile::ClearChargingProfileRequest, protocol_error::ProtocolError,
};

use crate::v16::{
    interface::{Database, Secc},
    cp::ChargePointCore,
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub(crate) fn clear_charging_profile_ocpp(
        &mut self,
        unique_id: String,
        _req: ClearChargingProfileRequest,
    ) {
        self.send_error(unique_id, ProtocolError::NotImplemented);
    }
}
