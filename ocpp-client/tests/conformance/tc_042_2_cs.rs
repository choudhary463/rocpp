use ocpp_core::v16::{
    messages::{
        get_local_list_version::{GetLocalListVersionRequest, GetLocalListVersionResponse},
        send_local_list::{SendLocalListRequest, SendLocalListResponse},
    },
    types::{UpdateStatus, UpdateType},
};

use crate::{
    state::reusable_states::{BootState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(SendLocalListRequest {
            list_version: 1,
            local_authorization_list: None,
            update_type: UpdateType::Full
        }),
        await_ws_msg(SendLocalListResponse {
            status: UpdateStatus::Accepted
        }),
        call(GetLocalListVersionRequest {}),
        await_ws_msg(GetLocalListVersionResponse { list_version: 0 })
    );

    chain
        .run(15, vec![("LocalAuthListEnabled", "true")], None)
        .await;
}
