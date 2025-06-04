use rocpp_core::v16::{
    messages::{
        start_transaction::{StartTransactionRequest, StartTransactionResponse},
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
    },
    types::{AuthorizationStatus, ChargePointStatus, IdTagInfo},
};

use crate::{
    state::reusable_states::{IdTagCachedState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let transaction_id = 1;
    let id_tag = format!("1234");
    let id_tag_info = IdTagInfo {
        expiry_date: None,
        parent_id_tag: None,
        status: AuthorizationStatus::Accepted,
    };

    let chain = test_chain!(
        IdTagCachedState::default(
            num_connectors,
            connector_id,
            id_tag.clone(),
            id_tag_info.clone()
        )
        .get_test_chain(),
        present_id_tag(connector_id, id_tag.clone()),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Preparing
        }),
        respond(StatusNotificationResponse {}),
        plug(connector_id),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Charging
        }),
        respond(StatusNotificationResponse {}),
        await_ws_msg(StartTransactionRequest {
            connector_id: connector_id,
            id_tag: id_tag.clone(),
            reservation_id: None
        }),
        respond(StartTransactionResponse {
            id_tag_info,
            transaction_id
        }),
        any_order(2),
    );

    chain
        .run(
            15,
            vec![
                ("AuthorizationCacheEnabled", "true"),
                ("LocalPreAuthorize", "true"),
            ],
            None,
        )
        .await;
}
