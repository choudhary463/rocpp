use rocpp_core::v16::{
    messages::{
        get_local_list_version::{GetLocalListVersionRequest, GetLocalListVersionResponse},
        send_local_list::{SendLocalListRequest, SendLocalListResponse},
    },
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

    let auth_data_differential1 = vec![AuthorizationData {
        id_tag: format!("2345"),
        id_tag_info: None,
    }];

    let auth_data_differential2 = vec![AuthorizationData {
        id_tag: format!("2345"),
        id_tag_info: Some(IdTagInfo {
            expiry_date: None,
            parent_id_tag: None,
            status: AuthorizationStatus::Accepted,
        }),
    }];

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(SendLocalListRequest {
            list_version: 2,
            local_authorization_list: Some(auth_data_full),
            update_type: UpdateType::Full
        }),
        await_ws_msg(SendLocalListResponse {
            status: UpdateStatus::Accepted
        }),
        call(GetLocalListVersionRequest {}),
        await_ws_msg(GetLocalListVersionResponse { list_version: 2 }),
        call(SendLocalListRequest {
            list_version: 5,
            local_authorization_list: Some(auth_data_differential1),
            update_type: UpdateType::Differential
        }),
        await_ws_msg(SendLocalListResponse {
            status: UpdateStatus::Accepted
        }),
        call(GetLocalListVersionRequest {}),
        await_ws_msg(GetLocalListVersionResponse { list_version: 5 }),
        call(SendLocalListRequest {
            list_version: 4,
            local_authorization_list: Some(auth_data_differential2),
            update_type: UpdateType::Differential
        }),
        await_ws_msg(SendLocalListResponse {
            status: UpdateStatus::VersionMismatch
        }),
        call(GetLocalListVersionRequest {}),
        await_ws_msg(GetLocalListVersionResponse { list_version: 5 }),
    );

    chain
        .run(15, vec![("LocalAuthListEnabled", "true")], None)
        .await;
}
