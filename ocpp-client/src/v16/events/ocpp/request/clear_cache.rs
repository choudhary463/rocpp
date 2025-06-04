use alloc::string::String;
use rocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::clear_cache::{ClearCacheRequest, ClearCacheResponse},
        types::ClearCacheStatus,
    },
};

use crate::v16::{cp::ChargePoint, interfaces::ChargePointInterface};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn clear_cache_ocpp(&mut self, unique_id: String, _req: ClearCacheRequest) {
        let status = if !self.configs.authorization_cache_enabled.value {
            ClearCacheStatus::Rejected
        } else {
            self.interface.db_clear_cache().await;
            ClearCacheStatus::Accepted
        };
        let payload = ClearCacheResponse { status };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode()).await;
    }
}
