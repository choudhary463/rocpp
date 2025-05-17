use ocpp_core::v16::{
    messages::{
        remote_stop_transaction::{RemoteStopTransactionRequest, RemoteStopTransactionResponse},
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
        stop_transaction::{StopTransactionRequest, StopTransactionResponse},
    },
    types::{ChargePointStatus, Reason, RemoteStartStopStatus},
};

use crate::{
    state::reusable_states::{ChargingState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let transaction_id = 1;
    let id_tag = format!("1234");

    let chain = test_chain!(
        ChargingState::default(num_connectors, connector_id, transaction_id, id_tag.clone())
            .get_test_chain(),
        call(RemoteStopTransactionRequest { transaction_id }),
        await_ws_msg(RemoteStopTransactionResponse {
            status: RemoteStartStopStatus::Accepted
        }),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Finishing
        }),
        respond(StatusNotificationResponse {}),
        await_ws_msg(StopTransactionRequest {
            transaction_id: transaction_id,
            id_tag: None,
            reason: Some(Reason::Remote)
        }),
        respond(StopTransactionResponse { id_tag_info: None }),
        any_order(2),
    );

    chain.run(15, vec![], None).await;
}
