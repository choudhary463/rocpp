use chrono::{Months, Utc};
use rocpp_core::v16::{
    messages::{
        authorize::{AuthorizeRequest, AuthorizeResponse},
        remote_start_transaction::{RemoteStartTransactionRequest, RemoteStartTransactionResponse},
        reserve_now::{ReserveNowRequest, ReserveNowResponse},
        start_transaction::{StartTransactionRequest, StartTransactionResponse},
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
    },
    types::{
        AuthorizationStatus, ChargePointStatus, IdTagInfo, RemoteStartStopStatus, ReservationStatus,
    },
};

use crate::{
    state::reusable_states::{BootState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let transaction_id = 1;
    let id_tag = format!("1234");

    let id_tag_info = IdTagInfo {
        expiry_date: None,
        parent_id_tag: None,
        status: AuthorizationStatus::Accepted,
    };

    let reservation_expiry_date = Utc::now().checked_add_months(Months::new(1)).unwrap();
    let reservation_id = 1;

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(ReserveNowRequest {
            connector_id,
            expiry_date: reservation_expiry_date,
            id_tag: id_tag.clone(),
            parent_id_tag: None,
            reservation_id
        }),
        await_ws_msg(ReserveNowResponse {
            status: ReservationStatus::Accepted
        }),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Reserved
        }),
        respond(StatusNotificationResponse {}),
        call(RemoteStartTransactionRequest {
            connector_id: Some(connector_id),
            id_tag: id_tag.clone(),
            charging_profile: None
        }),
        await_ws_msg(RemoteStartTransactionResponse {
            status: RemoteStartStopStatus::Accepted
        }),
        await_ws_msg(AuthorizeRequest {
            id_tag: id_tag.clone()
        }),
        respond(AuthorizeResponse {
            id_tag_info: id_tag_info.clone()
        }),
        optional(1),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Preparing
        }),
        respond(StatusNotificationResponse {}),
        plug(connector_id),
        await_ws_msg(StartTransactionRequest {
            connector_id: connector_id,
            id_tag: id_tag,
            reservation_id: Some(reservation_id)
        }),
        respond(StartTransactionResponse {
            id_tag_info: id_tag_info,
            transaction_id: transaction_id
        }),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Charging
        }),
        respond(StatusNotificationResponse {}),
        any_order(2)
    );

    chain.run(15, vec![], None).await;
}
