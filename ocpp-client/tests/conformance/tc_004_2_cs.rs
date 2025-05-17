use ocpp_core::v16::{
    messages::status_notification::{StatusNotificationRequest, StatusNotificationResponse},
    types::ChargePointStatus,
};

use crate::{
    state::reusable_states::{AuthorizeState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let id_tag = format!("1234");
    let minimum_status_duration = 4;

    let chain = test_chain!(
        AuthorizeState::default(num_connectors, connector_id, id_tag).get_test_chain(),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Available
        }),
        respond(StatusNotificationResponse {}),
        with_timing(minimum_status_duration * 1000 - 20, 40),
    );

    chain
        .run(
            15,
            vec![(
                "ConnectionTimeOut",
                minimum_status_duration.to_string().as_str(),
            )],
            None,
        )
        .await;
}
