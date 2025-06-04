use rocpp_core::v16::messages::get_local_list_version::{
    GetLocalListVersionRequest, GetLocalListVersionResponse,
};

use crate::{
    state::reusable_states::{BootState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(GetLocalListVersionRequest {}),
        await_ws_msg(GetLocalListVersionResponse { list_version: -1 })
    );

    chain
        .run(15, vec![("LocalAuthListEnabled", "false")], None)
        .await;
}
