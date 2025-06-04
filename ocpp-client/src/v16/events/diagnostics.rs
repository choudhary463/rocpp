use rocpp_core::v16::types::DiagnosticsStatus;

use crate::v16::{
    cp::ChargePoint,
    interfaces::{ChargePointInterface, DiagnosticsResponse},
    state_machine::diagnostics::DiagnosticsState,
};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn handle_diagnostics_response(&mut self, upload_status: DiagnosticsResponse) {
        if let DiagnosticsState::Uploading(mut t) =
            core::mem::replace(&mut self.diagnostics_state, DiagnosticsState::Idle)
        {
            if matches!(upload_status, DiagnosticsResponse::Success) {
                self.send_diagnostics_status_notification(DiagnosticsStatus::Uploaded)
                    .await;
            } else {
                t.retry_left -= 1;
                self.diagnostics_state = DiagnosticsState::Uploading(t);
                self.try_diagnostrics_upload().await;
            }
        } else {
            unreachable!();
        }
    }
}
