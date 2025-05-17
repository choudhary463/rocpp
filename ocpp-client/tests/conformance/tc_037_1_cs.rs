use ocpp_core::v16::{
    messages::{
        send_local_list::{SendLocalListRequest, SendLocalListResponse},
        start_transaction::{StartTransactionRequest, StartTransactionResponse},
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
    },
    types::{
        AuthorizationData, AuthorizationStatus, ChargePointStatus, IdTagInfo, UpdateStatus,
        UpdateType,
    },
};

use crate::{
    state::reusable_states::{ConnectionState, IdTagCachedState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let transaction_id = 1;
    let list_version = 1;
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
        call(SendLocalListRequest {
            list_version,
            local_authorization_list: Some(vec![AuthorizationData {
                id_tag: id_tag.clone(),
                id_tag_info: Some(id_tag_info.clone())
            }]),
            update_type: UpdateType::Full
        }),
        await_ws_msg(SendLocalListResponse {
            status: UpdateStatus::Accepted
        }),
        close_connection(),
        await_disconnection(),
        plug(connector_id),
        present_id_tag(connector_id, id_tag.clone()),
        restore_connection(),
        merge(ConnectionState::default().get_test_chain()),
        await_ws_msg(StartTransactionRequest {
            connector_id: connector_id,
            id_tag: id_tag
        }),
        respond(StartTransactionResponse {
            id_tag_info: id_tag_info.clone(),
            transaction_id: transaction_id
        }),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Charging
        }),
        respond(StatusNotificationResponse {}),
    );

    chain
        .run(
            15,
            vec![
                ("LocalAuthorizeOffline", "true"),
                ("LocalAuthListEnabled", "true"),
                ("AuthorizationCacheEnabled", "true"),
                ("AllowOfflineTxForUnknownId", "true"),
            ],
            None,
        )
        .await;
}
