use rocpp_core::v16::{
    messages::send_local_list::{SendLocalListRequest, SendLocalListResponse},
    types::{AuthorizationData, AuthorizationStatus, IdTagInfo, UpdateStatus, UpdateType},
};

use crate::{
    state::reusable_states::{BootState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let auth_data_full = vec![
        AuthorizationData {
            id_tag: format!("1234"),
            id_tag_info: Some(IdTagInfo {
                expiry_date: None,
                parent_id_tag: None,
                status: AuthorizationStatus::Accepted,
            }),
        },
        AuthorizationData {
            id_tag: format!("2345"),
            id_tag_info: Some(IdTagInfo {
                expiry_date: None,
                parent_id_tag: None,
                status: AuthorizationStatus::Invalid,
            }),
        },
    ];

    let auth_data_differential = vec![AuthorizationData {
        id_tag: format!("2345"),
        id_tag_info: None,
    }];

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(SendLocalListRequest {
            list_version: 1,
            local_authorization_list: Some(auth_data_full),
            update_type: UpdateType::Full
        }),
        await_ws_msg(SendLocalListResponse {
            status: UpdateStatus::Accepted
        }),
        call(SendLocalListRequest {
            list_version: 2,
            local_authorization_list: Some(auth_data_differential),
            update_type: UpdateType::Differential
        }),
        await_ws_msg(SendLocalListResponse {
            status: UpdateStatus::Accepted
        }),
    );

    chain
        .run(15, vec![("LocalAuthListEnabled", "true")], None)
        .await;
}
