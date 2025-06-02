use alloc::string::String;
use ocpp_core::v16::{
    messages::clear_charging_profile::ClearChargingProfileRequest, protocol_error::ProtocolError,
};

use crate::v16::{
    drivers::{database::Database, hardware_interface::HardwareInterface},
    cp::core::ChargePointCore,
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn clear_charging_profile_ocpp(
        &mut self,
        unique_id: String,
        _req: ClearChargingProfileRequest,
    ) {
        self.send_error(unique_id, ProtocolError::NotImplemented);
    }
}
