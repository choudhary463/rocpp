use alloc::string::String;
use ocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::data_transfer::{DataTransferRequest, DataTransferResponse},
        types::DataTransferStatus,
    },
};

use crate::v16::{
    drivers::{database::Database, hardware_interface::HardwareInterface},
    cp::core::ChargePointCore,
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn data_transfer_ocpp(&mut self, unique_id: String, _req: DataTransferRequest) {
        let payload = DataTransferResponse {
            status: DataTransferStatus::Rejected,
            data: None,
        };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode());
    }
}
