use ocpp_core::v16::{
    messages::{
        diagnostics_status_notification::{
            DiagnosticsStatusNotificationRequest, DiagnosticsStatusNotificationResponse,
        },
        get_diagnostics::{GetDiagnosticsRequest, GetDiagnosticsResponse},
    },
    types::DiagnosticsStatus,
};

use crate::{
    state::reusable_states::{BootState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let location = format!("valid_location");

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(GetDiagnosticsRequest {
            location,
            retries: None,
            start_time: None,
            stop_time: None,
            retry_interval: None
        }),
        await_ws_msg(GetDiagnosticsResponse {}),
        await_ws_msg(DiagnosticsStatusNotificationRequest {
            status: DiagnosticsStatus::Uploading
        }),
        respond(DiagnosticsStatusNotificationResponse {}),
        await_ws_msg(DiagnosticsStatusNotificationRequest {
            status: DiagnosticsStatus::Uploaded
        }),
        respond(DiagnosticsStatusNotificationResponse {})
    );

    chain.run(15, vec![], None).await;
}
