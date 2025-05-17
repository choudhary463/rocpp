use ocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::clear_cache::{ClearCacheRequest, ClearCacheResponse},
        types::ClearCacheStatus,
    },
};

use crate::v16::{
    interface::{Database, Secc},
    state_machine::core::ChargePointCore,
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn clear_cache_ocpp(&mut self, unique_id: String, _req: ClearCacheRequest) {
        let status = if !self.configs.authorization_cache_enabled.value {
            ClearCacheStatus::Rejected
        } else {
            let cached_id_tags = self.authorization_cache.drain().map(|f| f.0).collect();
            self.cache_usage_order.clear();
            self.db.db_delete_cache(cached_id_tags);
            ClearCacheStatus::Accepted
        };
        let payload = ClearCacheResponse { status };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode());
    }
}
