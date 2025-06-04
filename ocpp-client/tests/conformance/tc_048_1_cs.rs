use chrono::{Months, Utc};
use rocpp_core::v16::{
    messages::{
        reserve_now::{ReserveNowRequest, ReserveNowResponse},
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
    },
    types::{ChargePointStatus, ReservationStatus},
};

use crate::{
    state::reusable_states::{BootState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let id_tag = format!("1234");

    let reservation_expiry_date = Utc::now().checked_add_months(Months::new(1)).unwrap();
    let reservation_id = 1;

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        faulty(connector_id),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Faulted
        }),
        respond(StatusNotificationResponse {}),
        call(ReserveNowRequest {
            connector_id,
            expiry_date: reservation_expiry_date,
            id_tag: id_tag.clone(),
            parent_id_tag: None,
            reservation_id
        }),
        await_ws_msg(ReserveNowResponse {
            status: ReservationStatus::Faulted
        })
    );

    chain.run(15, vec![], None).await;
}
