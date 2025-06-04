use chrono::Utc;
use rocpp_core::v16::{
    messages::{
        diagnostics_status_notification::{
            DiagnosticsStatusNotificationRequest, DiagnosticsStatusNotificationResponse,
        },
        firmware_status_notification::{
            FirmwareStatusNotificationRequest, FirmwareStatusNotificationResponse,
        },
        heart_beat::{HeartbeatRequest, HeartbeatResponse},
        meter_values::{MeterValuesRequest, MeterValuesResponse},
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
        trigger_message::{TriggerMessageRequest, TriggerMessageResponse},
    },
    types::{
        ChargePointStatus, DiagnosticsStatus, FirmwareStatus, MessageTrigger, ReadingContext,
        TriggerMessageStatus,
    },
};

use crate::{
    state::{
        reusable_states::{get_sampled_value, validate_meter_values, BootState, ReusableState},
        ws_recv::AfterValidation,
    },
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;

    let mut chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(TriggerMessageRequest {
            connector_id: Some(connector_id),
            requested_message: MessageTrigger::MeterValues
        }),
        await_ws_msg(TriggerMessageResponse {
            status: TriggerMessageStatus::Accepted
        })
    );

    chain = chain
        .await_ws_msg::<MeterValuesRequest>()
        .done_custom(move |t| {
            let values = match get_sampled_value("MeterValue", t, connector_id, None) {
                Ok(v) => v,
                Err(e) => return AfterValidation::Failed(e),
            };
            let _ = match validate_meter_values(
                "MeterValue",
                values,
                None,
                ReadingContext::Trigger,
                None,
                0,
                0,
            ) {
                Ok(ts) => ts,
                Err(e) => return AfterValidation::Failed(e),
            };
            AfterValidation::NextDefault
        })
        .respond(MeterValuesResponse {})
        .with_timing(0, 20);

    chain = test_chain!(
        chain,
        call(TriggerMessageRequest {
            connector_id: None,
            requested_message: MessageTrigger::Heartbeat
        }),
        await_ws_msg(TriggerMessageResponse {
            status: TriggerMessageStatus::Accepted
        }),
        await_ws_msg(HeartbeatRequest {}),
        respond_with_now(HeartbeatResponse {
            current_time: Utc::now()
        }),
        with_timing(0, 20),
        call(TriggerMessageRequest {
            connector_id: Some(connector_id),
            requested_message: MessageTrigger::StatusNotification
        }),
        await_ws_msg(TriggerMessageResponse {
            status: TriggerMessageStatus::Accepted
        }),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Available
        }),
        respond(StatusNotificationResponse {}),
        with_timing(0, 20),
        call(TriggerMessageRequest {
            connector_id: None,
            requested_message: MessageTrigger::DiagnosticsStatusNotification
        }),
        await_ws_msg(TriggerMessageResponse {
            status: TriggerMessageStatus::Accepted
        }),
        await_ws_msg(DiagnosticsStatusNotificationRequest {
            status: DiagnosticsStatus::Idle
        }),
        respond(DiagnosticsStatusNotificationResponse {}),
        with_timing(0, 20),
        call(TriggerMessageRequest {
            connector_id: None,
            requested_message: MessageTrigger::FirmwareStatusNotification
        }),
        await_ws_msg(TriggerMessageResponse {
            status: TriggerMessageStatus::Accepted
        }),
        await_ws_msg(FirmwareStatusNotificationRequest {
            status: FirmwareStatus::Idle
        }),
        respond(FirmwareStatusNotificationResponse {}),
        with_timing(0, 20)
    );
    chain.run(15, vec![], None).await;
}
