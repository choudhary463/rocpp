use rocpp_core::v16::{
    messages::{
        authorize::{AuthorizeRequest, AuthorizeResponse},
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
    },
    types::{AuthorizationStatus, ChargePointStatus, IdTagInfo},
};

use crate::{
    state::reusable_states::{BootState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let id_tag = format!("1234");

    let id_tag_info = IdTagInfo {
        expiry_date: None,
        parent_id_tag: None,
        status: AuthorizationStatus::Invalid,
    };

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        present_id_tag(connector_id, id_tag.clone()),
        await_ws_msg(AuthorizeRequest {
            id_tag: id_tag.clone()
        }),
        respond(AuthorizeResponse {
            id_tag_info: id_tag_info
        }),
        plug(connector_id),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Preparing
        }),
        respond(StatusNotificationResponse {}),
        await_timeout()
    );

    chain.run(15, vec![], None).await;
}
