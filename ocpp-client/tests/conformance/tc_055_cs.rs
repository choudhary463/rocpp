use ocpp_core::v16::{
    messages::trigger_message::{TriggerMessageRequest, TriggerMessageResponse},
    types::{MessageTrigger, TriggerMessageStatus},
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
        call(TriggerMessageRequest {
            connector_id: Some(connector_id),
            requested_message: MessageTrigger::MeterValues
        }),
        await_ws_msg(TriggerMessageResponse {
            status: TriggerMessageStatus::Rejected
        })
    );
    chain.run(15, vec![], None).await;
}
