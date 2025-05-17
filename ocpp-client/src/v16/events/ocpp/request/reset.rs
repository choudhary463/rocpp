use ocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::reset::{ResetRequest, ResetResponse},
        types::ResetStatus,
    },
};

use crate::v16::{
    interface::{Database, Secc},
    state_machine::core::ChargePointCore,
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn reset_ocpp(&mut self, unique_id: String, req: ResetRequest) {
        let payload = ResetResponse {
            status: ResetStatus::Accepted,
        };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode());
        self.reset(req.kind, None);
    }
}
