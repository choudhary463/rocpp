use rocpp_core::v16::{
    messages::{
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
        stop_transaction::{StopTransactionRequest, StopTransactionResponse},
        unlock_connector::{UnlockConnectorRequest, UnlockConnectorResponse},
    },
    types::{ChargePointStatus, Reason, UnlockStatus},
};

use crate::{
    state::reusable_states::{ChargingState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let transaction_id = 1;

    let id_tag = format!("1234");

    let chain = test_chain!(
        ChargingState::default(num_connectors, connector_id, transaction_id, id_tag)
            .get_test_chain(),
        unplug(connector_id),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Available
        }),
        respond(StatusNotificationResponse {}),
        await_ws_msg(StopTransactionRequest {
            transaction_id: transaction_id,
            id_tag: None,
            reason: Some(Reason::EVDisconnected)
        }),
        respond(StopTransactionResponse { id_tag_info: None }),
        any_order(2),
        call(UnlockConnectorRequest {
            connector_id: connector_id
        }),
        await_ws_msg(UnlockConnectorResponse {
            status: UnlockStatus::NotSupported
        }),
    );

    chain
        .run(
            15,
            vec![
                ("MinimumStatusDuration", "0"),
                ("StopTransactionOnEVSideDisconnect", "true"),
            ],
            None,
        )
        .await;
}
