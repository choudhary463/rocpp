use alloc::string::String;
use rocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::reset::{ResetRequest, ResetResponse},
        types::ResetStatus,
    },
};

use crate::v16::{cp::ChargePoint, interfaces::ChargePointInterface};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn reset_ocpp(&mut self, unique_id: String, req: ResetRequest) {
        let payload = ResetResponse {
            status: ResetStatus::Accepted,
        };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode()).await;
        self.reset(req.kind, None).await;
    }
}
