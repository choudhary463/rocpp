use ocpp_core::v16::{
    messages::clear_charging_profile::ClearChargingProfileRequest, protocol_error::ProtocolError,
};

use crate::v16::{
    interface::{Database, Secc},
    state_machine::core::ChargePointCore,
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn clear_charging_profile_ocpp(
        &mut self,
        unique_id: String,
        _req: ClearChargingProfileRequest,
    ) {
        self.send_error(unique_id, ProtocolError::NotImplemented);
    }
}
