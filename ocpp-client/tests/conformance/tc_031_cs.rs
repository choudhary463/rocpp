use rocpp_core::v16::{
    messages::unlock_connector::{UnlockConnectorRequest, UnlockConnectorResponse},
    types::UnlockStatus,
};

use crate::{
    state::reusable_states::{BootState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 3;

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(UnlockConnectorRequest {
            connector_id: connector_id
        }),
        await_ws_msg(UnlockConnectorResponse {
            status: UnlockStatus::NotSupported
        })
    );

    chain.run(15, vec![], None).await;
}
