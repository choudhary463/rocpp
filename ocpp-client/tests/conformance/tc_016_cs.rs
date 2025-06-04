use rocpp_core::v16::{
    messages::{
        reset::{ResetRequest, ResetResponse},
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
        stop_transaction::{StopTransactionRequest, StopTransactionResponse},
    },
    types::{ChargePointStatus, Reason, ResetStatus, ResetType},
};

use crate::{
    state::reusable_states::{BootState, ChargingState, ConnectionState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let transaction_id = 1;
    let id_tag = format!("1234");

    let chain = test_chain!(
        ChargingState::default(num_connectors, connector_id, transaction_id, id_tag)
            .get_test_chain(),
        call(ResetRequest {
            kind: ResetType::Soft
        }),
        await_ws_msg(ResetResponse {
            status: ResetStatus::Accepted
        }),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Finishing
        }),
        respond(StatusNotificationResponse {}),
        await_ws_msg(StopTransactionRequest {
            transaction_id: transaction_id,
            id_tag: None,
            reason: Some(Reason::SoftReset)
        }),
        respond(StopTransactionResponse { id_tag_info: None }),
        any_order(2),
        merge(
            ConnectionState::default()
                .with_disconnection()
                .get_test_chain()
        ),
        merge(BootState::default(num_connectors).get_self_chain()),
        optional(2),
        await_timeout()
    );

    chain.run(15, vec![], None).await;
}
