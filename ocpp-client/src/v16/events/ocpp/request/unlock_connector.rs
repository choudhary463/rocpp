use alloc::string::String;
use ocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::unlock_connector::{UnlockConnectorRequest, UnlockConnectorResponse},
        types::UnlockStatus,
    },
};

use crate::v16::{
    drivers::{database::Database, hardware_interface::HardwareInterface},
    cp::core::ChargePointCore,
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn unlock_connector_ocpp(&mut self, unique_id: String, _req: UnlockConnectorRequest) {
        let payload = UnlockConnectorResponse {
            status: UnlockStatus::NotSupported,
        };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode());
    }
}
