use rocpp_core::v16::{
    messages::{
        authorize::{AuthorizeRequest, AuthorizeResponse},
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
        stop_transaction::{StopTransactionRequest, StopTransactionResponse},
    },
    types::{AuthorizationStatus, ChargePointStatus, IdTagInfo},
};

use crate::{
    state::reusable_states::{AuthorizeState, ChargingState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let transaction_id = 1;

    let id_tag1 = format!("1234");
    let id_tag1_info = IdTagInfo {
        expiry_date: None,
        parent_id_tag: Some(format!("parent")),
        status: AuthorizationStatus::Accepted,
    };

    let id_tag2 = format!("2345");
    let id_tag2_info = IdTagInfo {
        expiry_date: None,
        parent_id_tag: Some(format!("parent")),
        status: AuthorizationStatus::Accepted,
    };

    let chain = test_chain!(
        ChargingState::custom(
            AuthorizeState::custom_with_default_boot(
                num_connectors,
                connector_id,
                id_tag1.clone(),
                id_tag1_info.clone()
            ),
            transaction_id
        )
        .get_test_chain(),
        present_id_tag(connector_id, id_tag2.clone()),
        await_ws_msg(AuthorizeRequest {
            id_tag: id_tag2.clone()
        }),
        respond(AuthorizeResponse {
            id_tag_info: id_tag2_info.clone()
        }),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Finishing
        }),
        respond(StatusNotificationResponse {}),
        await_ws_msg(StopTransactionRequest {
            transaction_id: transaction_id,
            id_tag: Some(id_tag2.clone())
        }),
        respond(StopTransactionResponse { id_tag_info: None }),
        any_order(2),
    );

    chain.run(15, vec![], None).await;
}
