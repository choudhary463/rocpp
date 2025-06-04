use chrono::{Months, Utc};
use rocpp_core::v16::{
    messages::{
        authorize::{AuthorizeRequest, AuthorizeResponse},
        cancel_reservation::{CancelReservationRequest, CancelReservationResponse},
        reserve_now::{ReserveNowRequest, ReserveNowResponse},
        start_transaction::{StartTransactionRequest, StartTransactionResponse},
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
    },
    types::{
        AuthorizationStatus, CancelReservationStatus, ChargePointStatus, IdTagInfo,
        ReservationStatus,
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
    let id_tag1 = format!("1234");
    let id_tag2 = format!("2345");

    let id_tag2_info = IdTagInfo {
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
            id_tag: id_tag1.clone(),
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
        call(CancelReservationRequest { reservation_id }),
        await_ws_msg(CancelReservationResponse {
            status: CancelReservationStatus::Accepted
        }),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Available
        }),
        respond(StatusNotificationResponse {}),
        present_id_tag(connector_id, id_tag2.clone()),
        await_ws_msg(AuthorizeRequest { id_tag: id_tag2 }),
        respond(AuthorizeResponse {
            id_tag_info: id_tag2_info.clone()
        }),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Preparing
        }),
        respond(StatusNotificationResponse {}),
        plug(connector_id),
        await_ws_msg(StartTransactionRequest {
            connector_id: connector_id,
            id_tag: id_tag2,
            reservation_id: None
        }),
        respond(StartTransactionResponse {
            id_tag_info: id_tag2_info,
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
