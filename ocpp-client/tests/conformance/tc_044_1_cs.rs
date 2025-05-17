use chrono::Utc;
use ocpp_core::v16::{
    messages::{
        firmware_status_notification::{
            FirmwareStatusNotificationRequest, FirmwareStatusNotificationResponse,
        },
        update_firmware::{UpdateFirmwareRequest, UpdateFirmwareResponse},
    },
    types::{ChargePointStatus, FirmwareStatus},
};

use crate::{
    state::reusable_states::{get_all_connector_states, BootState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let location = format!("download_success:install:success");

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(UpdateFirmwareRequest {
            location,
            retries: None,
            retrieve_date: Utc::now(),
            retry_interval: None
        }),
        await_ws_msg(UpdateFirmwareResponse {}),
        merge(get_all_connector_states(vec![
            ChargePointStatus::Unavailable;
            num_connectors
        ])),
        optional(1),
        await_ws_msg(FirmwareStatusNotificationRequest {
            status: FirmwareStatus::Downloading
        }),
        respond(FirmwareStatusNotificationResponse {}),
        await_ws_msg(FirmwareStatusNotificationRequest {
            status: FirmwareStatus::Downloaded
        }),
        respond(FirmwareStatusNotificationResponse {}),
        await_ws_msg(FirmwareStatusNotificationRequest {
            status: FirmwareStatus::Installing
        }),
        respond(FirmwareStatusNotificationResponse {}),
        await_disconnection(),
        merge(BootState::default(num_connectors).get_test_chain()),
        pop(),
        await_ws_msg(FirmwareStatusNotificationRequest {
            status: FirmwareStatus::Installed
        }),
        respond(FirmwareStatusNotificationResponse {}),
        any_order(num_connectors + 1)
    );

    chain.run(15, vec![], None).await;
}
