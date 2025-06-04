use rocpp_core::v16::{
    messages::remote_start_transaction::{
        RemoteStartTransactionRequest, RemoteStartTransactionResponse,
    },
    types::RemoteStartStopStatus,
};

use crate::{
    state::reusable_states::{BootState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 0;
    let id_tag = format!("1234");

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(RemoteStartTransactionRequest {
            connector_id: Some(connector_id),
            id_tag,
            charging_profile: None
        }),
        await_ws_msg(RemoteStartTransactionResponse {
            status: RemoteStartStopStatus::Rejected
        })
    );

    chain.run(15, vec![], None).await;
}
