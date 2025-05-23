use ocpp_core::v16::types::DiagnosticsStatus;

use crate::v16::{
    cp::ChargePointCore, interface::{Database, Secc}, state_machine::{diagnostics::DiagnosticsState}, DiagnosticsResponse
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub(crate) fn handle_diagnostics_response_helper(&mut self, upload_status: DiagnosticsResponse) {
        if let DiagnosticsState::Uploading(mut t) =
            core::mem::replace(&mut self.diagnostics_state, DiagnosticsState::Idle)
        {
            if matches!(upload_status, DiagnosticsResponse::Success) {
                self.send_diagnostics_status_notification(DiagnosticsStatus::Uploaded);
            } else {
                t.retry_left -= 1;
                self.diagnostics_state = DiagnosticsState::Uploading(t);
                self.try_diagnostrics_upload();
            }
        } else {
            unreachable!();
        }
    }
}
