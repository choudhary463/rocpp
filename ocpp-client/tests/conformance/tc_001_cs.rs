use chrono::Utc;
use rocpp_core::v16::{
    messages::{
        boot_notification::{BootNotificationRequest, BootNotificationResponse},
        heart_beat::{HeartbeatRequest, HeartbeatResponse},
    },
    types::{ChargePointStatus, RegistrationStatus},
};

use crate::{
    state::reusable_states::{get_all_connector_states, ConnectionState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;

    let chain = test_chain!(
        ConnectionState::default().get_test_chain(),
        await_ws_msg(BootNotificationRequest {}),
        respond_with_now(BootNotificationResponse {
            current_time: Utc::now(),
            interval: 2,
            status: RegistrationStatus::Rejected
        }),
        await_ws_msg(BootNotificationRequest {}),
        respond_with_now(BootNotificationResponse {
            current_time: Utc::now(),
            interval: 1,
            status: RegistrationStatus::Accepted
        }),
        with_timing(2000, 20),
        merge(get_all_connector_states(vec![
            ChargePointStatus::Available;
            num_connectors
        ])),
        await_ws_msg(HeartbeatRequest {}),
        respond_with_now(HeartbeatResponse {
            current_time: Utc::now()
        }),
        with_timing(1000, 20),
        await_ws_msg(HeartbeatRequest {}),
        respond_with_now(HeartbeatResponse {
            current_time: Utc::now()
        }),
        with_timing(1000, 20),
    );

    chain.run(15, vec![], None).await;
}
