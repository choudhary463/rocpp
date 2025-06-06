use alloc::string::String;
use rocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::remote_stop_transaction::{
            RemoteStopTransactionRequest, RemoteStopTransactionResponse,
        },
        types::{Reason, RemoteStartStopStatus},
    },
};

use crate::v16::{cp::ChargePoint, interfaces::ChargePointInterface};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn remote_stop_transaction_ocpp(
        &mut self,
        unique_id: String,
        req: RemoteStopTransactionRequest,
    ) {
        let connector_id = self
            .active_local_transactions
            .iter()
            .position(|v| v.map(|f| f.1 == Some(req.transaction_id)).unwrap_or(false));

        let status = if connector_id.is_none() {
            RemoteStartStopStatus::Rejected
        } else {
            RemoteStartStopStatus::Accepted
        };
        let payload = RemoteStopTransactionResponse { status };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode()).await;

        if let Some(connector_id) = connector_id {
            self.stop_transaction(connector_id, None, Some(Reason::Remote))
                .await;
        }
    }
}
