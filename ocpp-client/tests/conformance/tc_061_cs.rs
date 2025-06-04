use rocpp_core::v16::{
    messages::{
        authorize::{AuthorizeRequest, AuthorizeResponse},
        clear_cache::{ClearCacheRequest, ClearCacheResponse},
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
    },
    types::{AuthorizationStatus, ChargePointStatus, ClearCacheStatus, IdTagInfo},
};

use crate::{
    state::reusable_states::{AuthorizeState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let id_tag = format!("1234");
    let id_tag_info = IdTagInfo {
        expiry_date: None,
        parent_id_tag: None,
        status: AuthorizationStatus::Accepted,
    };

    let chain = test_chain!(
        AuthorizeState::default(num_connectors, connector_id, id_tag.clone()).get_test_chain(),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Available
        }),
        respond(StatusNotificationResponse {}),
        call(ClearCacheRequest {}),
        await_ws_msg(ClearCacheResponse {
            status: ClearCacheStatus::Accepted
        }),
        present_id_tag(connector_id, id_tag.clone()),
        await_ws_msg(AuthorizeRequest {
            id_tag: id_tag.clone()
        }),
        respond(AuthorizeResponse {
            id_tag_info: id_tag_info.clone()
        }),
    );

    chain
        .run(
            15,
            vec![
                ("AuthorizationCacheEnabled", "true"),
                ("LocalPreAuthorize", "true"),
                ("ConnectionTimeOut", "4"),
            ],
            None,
        )
        .await;
}
