use alloc::string::String;
use rocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::messages::get_local_list_version::{
        GetLocalListVersionRequest, GetLocalListVersionResponse,
    },
};

use crate::v16::{cp::ChargePoint, interfaces::ChargePointInterface};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn local_list_version(&mut self) -> i32 {
        if self.configs.local_auth_list_enabled.value {
            if self.local_list_entries_count == 0 {
                0
            } else {
                self.interface.db_get_local_list_version().await.unwrap()
            }
        } else {
            -1
        }
    }
    pub(crate) async fn get_local_list_version_ocpp(
        &mut self,
        unique_id: String,
        _req: GetLocalListVersionRequest,
    ) {
        let list_version = self.local_list_version().await;
        let payload = GetLocalListVersionResponse { list_version };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode()).await;
    }
}
