use ocpp_core::v16::{
    messages::remote_start_transaction::{
        RemoteStartTransactionRequest, RemoteStartTransactionResponse,
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
    let id_tag1 = format!("1234");
    let id_tag2 = format!("2345");

    let chain = test_chain!(
        ChargingState::default(num_connectors, connector_id, transaction_id, id_tag1)
            .get_test_chain(),
        call(RemoteStartTransactionRequest {
            connector_id: Some(connector_id),
            id_tag: id_tag2,
            charging_profile: None
        }),
        await_ws_msg(RemoteStartTransactionResponse {
            status: RemoteStartStopStatus::Rejected
        })
    );

    chain.run(15, vec![], None).await;
}
