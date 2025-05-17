use ocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::unlock_connector::{UnlockConnectorRequest, UnlockConnectorResponse},
        types::UnlockStatus,
    },
};

use crate::v16::{
    interface::{Database, Secc},
    state_machine::core::ChargePointCore,
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn unlock_connector_ocpp(&mut self, unique_id: String, _req: UnlockConnectorRequest) {
        let payload = UnlockConnectorResponse {
            status: UnlockStatus::NotSupported,
        };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode());
    }
}
