use rocpp_core::v16::{
    messages::remote_stop_transaction::{
        RemoteStopTransactionRequest, RemoteStopTransactionResponse,
    },
    types::RemoteStartStopStatus,
};

use crate::{
    state::reusable_states::{ChargingState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let transaction_id = 1;
    let transaction_id_unknown = 2;
    let id_tag = format!("1234");

    let chain = test_chain!(
        ChargingState::default(num_connectors, connector_id, transaction_id, id_tag)
            .get_test_chain(),
        call(RemoteStopTransactionRequest {
            transaction_id: transaction_id_unknown
        }),
        await_ws_msg(RemoteStopTransactionResponse {
            status: RemoteStartStopStatus::Rejected
        })
    );

    chain.run(15, vec![], None).await;
}
