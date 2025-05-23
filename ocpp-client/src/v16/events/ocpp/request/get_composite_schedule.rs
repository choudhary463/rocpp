use alloc::string::String;
use ocpp_core::v16::{
    messages::get_composite_schedule::GetCompositeScheduleRequest, protocol_error::ProtocolError,
};

use crate::v16::{
    interface::{Database, Secc},
    cp::ChargePointCore,
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub(crate) fn get_composite_schedule_ocpp(
        &mut self,
        unique_id: String,
        _req: GetCompositeScheduleRequest,
    ) {
        self.send_error(unique_id, ProtocolError::NotImplemented);
    }
}
