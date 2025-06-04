use rocpp_core::v16::{
    messages::{
        reset::{ResetRequest, ResetResponse},
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
        stop_transaction::{StopTransactionRequest, StopTransactionResponse},
    },
    types::{ChargePointStatus, Reason, ResetStatus, ResetType},
};

use crate::{
    state::{
        reusable_states::{BootState, ChargingState, ReusableState},
        step::TestChain,
    },
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let transaction_id = 1;
    let id_tag = format!("1234");

    let base_dir = std::env::temp_dir();
    let db_dir = Some(base_dir.join("tc_015"));

    let chain1 = test_chain!(
        TestChain::new(),
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
        any_order(num_connectors + 1)
    );

    let chain2 = test_chain!(
        TestChain::new(),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Finishing
        }),
        respond(StatusNotificationResponse {}),
        await_ws_msg(StopTransactionRequest {
            transaction_id: transaction_id,
            id_tag: None,
            reason: Some(Reason::HardReset)
        }),
        respond(StopTransactionResponse { id_tag_info: None }),
        any_order(2),
        await_hard_reset(),
        spawn_new(15, vec![], db_dir.clone(), false),
        plug(connector_id),
        merge(
            BootState::default(num_connectors)
                .with_state(connector_id, ChargePointStatus::Preparing)
                .get_test_chain()
        ),
    );

    let chain = test_chain!(
        ChargingState::default(num_connectors, connector_id, transaction_id, id_tag)
            .get_test_chain(),
        call(ResetRequest {
            kind: ResetType::Hard
        }),
        await_ws_msg(ResetResponse {
            status: ResetStatus::Accepted
        }),
        merge_into_one(chain1),
        merge_into_one(chain2),
        either()
    );

    chain.run(15, vec![], db_dir).await;
}
