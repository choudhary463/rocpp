use rocpp_core::v16::{
    messages::{
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
        stop_transaction::{StopTransactionRequest, StopTransactionResponse},
    },
    types::{ChargePointStatus, Reason},
};

use crate::{
    state::reusable_states::{ChargingState, ConnectionState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let transaction_id = 1;
    let id_tag = format!("1234");
    let base_dir = std::env::temp_dir();
    let db_dir = Some(base_dir.join("tc_038"));

    let chain = test_chain!(
        ChargingState::default(num_connectors, connector_id, transaction_id, id_tag.clone())
            .get_test_chain(),
        close_connection(),
        await_disconnection(),
        present_id_tag(connector_id, id_tag),
        restore_connection(),
        merge(ConnectionState::default().get_test_chain()),
        await_ws_msg(StopTransactionRequest {
            transaction_id: transaction_id,
            reason: Some(Reason::Local)
        }),
        respond(StopTransactionResponse { id_tag_info: None }),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Finishing
        }),
        respond(StatusNotificationResponse {}),
        any_order(2)
    );

    chain.run(15, vec![], db_dir).await;
}
