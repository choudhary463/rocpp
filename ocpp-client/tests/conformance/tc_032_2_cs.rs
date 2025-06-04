use rocpp_core::v16::{
    messages::stop_transaction::{StopTransactionRequest, StopTransactionResponse},
    types::{ChargePointStatus, Reason},
};

use crate::{
    state::reusable_states::{BootState, ChargingState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let transaction_id = 1;
    let id_tag = format!("1234");

    let base_dir = std::env::temp_dir();
    let db_dir = Some(base_dir.join("tc_032_2"));

    let chain = test_chain!(
        ChargingState::default(num_connectors, connector_id, transaction_id, id_tag)
            .get_test_chain(),
        cut_power(),
        await_hard_reset(),
        spawn_new(15, vec![], db_dir.clone(), false),
        plug(connector_id),
        merge(
            BootState::default(num_connectors)
                .with_state(connector_id, ChargePointStatus::Preparing)
                .get_test_chain()
        ),
        pop(),
        await_ws_msg(StopTransactionRequest {
            transaction_id: transaction_id,
            id_tag: None,
            reason: Some(Reason::PowerLoss)
        }),
        respond(StopTransactionResponse { id_tag_info: None }),
        any_order(num_connectors + 1),
    );

    chain.run(15, vec![], db_dir).await;
}
