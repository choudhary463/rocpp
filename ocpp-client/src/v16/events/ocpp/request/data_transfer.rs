use alloc::string::String;
use rocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::data_transfer::{DataTransferRequest, DataTransferResponse},
        types::DataTransferStatus,
    },
};

use crate::v16::{cp::ChargePoint, interfaces::ChargePointInterface};


impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn data_transfer_ocpp(&mut self, unique_id: String, _req: DataTransferRequest) {
        let payload = DataTransferResponse {
            status: DataTransferStatus::Rejected,
            data: None,
        };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode()).await;
    }
}
