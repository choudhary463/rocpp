use rocpp_core::v16::{
    messages::send_local_list::{SendLocalListRequest, SendLocalListResponse},
    types::{AuthorizationData, UpdateStatus, UpdateType},
};

use crate::{
    state::reusable_states::{BootState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let id_tag = format!("1234");

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(SendLocalListRequest {
            list_version: 2,
            local_authorization_list: Some(vec![AuthorizationData {
                id_tag: id_tag,
                id_tag_info: None
            }]),
            update_type: UpdateType::Full
        }),
        await_ws_msg(SendLocalListResponse {
            status: UpdateStatus::Failed
        }),
    );

    chain
        .run(15, vec![("LocalAuthListEnabled", "true")], None)
        .await;
}
