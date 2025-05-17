use ocpp_core::v16::{
    messages::change_configuration::{ChangeConfigurationRequest, ChangeConfigurationResponse},
    types::ConfigurationStatus,
};

use crate::{
    state::reusable_states::{BootState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(ChangeConfigurationRequest {
            key: format!("Testing"),
            value: format!("true")
        }),
        await_ws_msg(ChangeConfigurationResponse {
            status: ConfigurationStatus::NotSupported
        })
    );

    chain.run(15, vec![], None).await;
}
