use ocpp_core::v16::{
    messages::status_notification::{StatusNotificationRequest, StatusNotificationResponse},
    types::ChargePointStatus,
};

use crate::{
    state::reusable_states::{BootState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let id_tag = format!("1234");

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        faulty(connector_id),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Faulted
        }),
        respond(StatusNotificationResponse {}),
        present_id_tag(connector_id, id_tag),
        await_timeout()
    );

    chain.run(15, vec![], None).await;
}
