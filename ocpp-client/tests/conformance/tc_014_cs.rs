use rocpp_core::v16::{
    messages::{
        change_availability::{ChangeAvailabilityRequest, ChangeAvailabilityResponse},
        reset::{ResetRequest, ResetResponse},
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
    },
    types::{AvailabilityStatus, AvailabilityType, ChargePointStatus, ResetStatus, ResetType},
};

use crate::{
    state::reusable_states::{BootState, ConnectionState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(ChangeAvailabilityRequest {
            connector_id,
            kind: AvailabilityType::Inoperative
        }),
        await_ws_msg(ChangeAvailabilityResponse {
            status: AvailabilityStatus::Accepted
        }),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Unavailable
        }),
        respond(StatusNotificationResponse {}),
        call(ResetRequest {
            kind: ResetType::Soft
        }),
        await_ws_msg(ResetResponse {
            status: ResetStatus::Accepted
        }),
        merge(
            ConnectionState::default()
                .with_disconnection()
                .get_self_chain()
        ),
        merge(
            BootState::default(num_connectors)
                .with_state(connector_id, ChargePointStatus::Unavailable)
                .get_self_chain()
        ),
        optional(2),
        await_timeout(),
        call(ChangeAvailabilityRequest {
            connector_id,
            kind: AvailabilityType::Operative
        }),
        await_ws_msg(ChangeAvailabilityResponse {
            status: AvailabilityStatus::Accepted
        }),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Available
        }),
        respond(StatusNotificationResponse {})
    );

    chain.run(15, vec![], None).await;
}
