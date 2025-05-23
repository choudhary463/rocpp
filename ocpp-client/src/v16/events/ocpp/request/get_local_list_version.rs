use alloc::string::String;
use ocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::messages::get_local_list_version::{
        GetLocalListVersionRequest, GetLocalListVersionResponse,
    },
};

use crate::v16::{
    interface::{Database, Secc},
    cp::ChargePointCore,
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub(crate) fn get_local_list_version_ocpp(
        &mut self,
        unique_id: String,
        _req: GetLocalListVersionRequest,
    ) {
        let list_version = if self.configs.local_auth_list_enabled.value {
            self.local_list_version
        } else {
            -1
        };
        let payload = GetLocalListVersionResponse { list_version };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode());
    }
}
