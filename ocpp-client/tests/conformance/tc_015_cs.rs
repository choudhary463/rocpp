use ocpp_core::v16::{
    messages::{
        reset::{ResetRequest, ResetResponse},
        stop_transaction::{StopTransactionRequest, StopTransactionResponse},
    },
    types::{ChargePointStatus, Reason, ResetStatus, ResetType},
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
    let db_dir = Some(base_dir.join("tc_015"));

    let chain = test_chain!(
        ChargingState::default(num_connectors, connector_id, transaction_id, id_tag)
            .get_test_chain(),
        call(ResetRequest {
            kind: ResetType::Hard
        }),
        await_ws_msg(ResetResponse {
            status: ResetStatus::Accepted
        }),
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
            reason: Some(Reason::HardReset)
        }),
        respond(StopTransactionResponse { id_tag_info: None }),
        any_order(num_connectors + 1),
    );

    chain.run(15, vec![], db_dir).await;
}
