use alloc::string::String;
use ocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::reset::{ResetRequest, ResetResponse},
        types::ResetStatus,
    },
};

use crate::v16::{
    drivers::{database::Database, hardware_interface::HardwareInterface},
    cp::core::ChargePointCore,
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn reset_ocpp(&mut self, unique_id: String, req: ResetRequest) {
        let payload = ResetResponse {
            status: ResetStatus::Accepted,
        };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode());
        self.reset(req.kind, None);
    }
}
