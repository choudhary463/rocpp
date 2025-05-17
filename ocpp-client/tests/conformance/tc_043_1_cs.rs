use ocpp_core::v16::{
    messages::send_local_list::{SendLocalListRequest, SendLocalListResponse},
    types::{AuthorizationData, AuthorizationStatus, IdTagInfo, UpdateStatus, UpdateType},
};

use crate::{
    state::reusable_states::{BootState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let id_tag = format!("1234");
    let id_tag_info = IdTagInfo {
        expiry_date: None,
        parent_id_tag: None,
        status: AuthorizationStatus::Accepted,
    };

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(SendLocalListRequest {
            list_version: 1,
            local_authorization_list: Some(vec![AuthorizationData {
                id_tag: id_tag,
                id_tag_info: Some(id_tag_info)
            }]),
            update_type: UpdateType::Full
        }),
        await_ws_msg(SendLocalListResponse {
            status: UpdateStatus::NotSupported
        }),
    );

    chain
        .run(15, vec![("LocalAuthListEnabled", "false")], None)
        .await;
}
