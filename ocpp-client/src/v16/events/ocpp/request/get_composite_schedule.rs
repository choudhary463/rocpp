use alloc::string::String;
use ocpp_core::v16::{
    messages::get_composite_schedule::GetCompositeScheduleRequest, protocol_error::ProtocolError,
};

use crate::v16::{
    drivers::{database::Database, hardware_interface::HardwareInterface},
    cp::core::ChargePointCore,
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn get_composite_schedule_ocpp(
        &mut self,
        unique_id: String,
        _req: GetCompositeScheduleRequest,
    ) {
        self.send_error(unique_id, ProtocolError::NotImplemented);
    }
}
