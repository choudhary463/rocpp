use ocpp_core::v16::types::DiagnosticsStatus;

use crate::v16::{
    interface::{Database, Secc},
    services::timeout::TimerId,
    state_machine::{core::ChargePointCore, diagnostics::DiagnosticsState},
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn handle_diagnostics_response(&mut self, upload_status: bool) {
        if let DiagnosticsState::Uploading(mut t) =
            std::mem::replace(&mut self.diagnostics_state, DiagnosticsState::Idle)
        {
            self.remove_timeout(TimerId::Diagnostics);
            if upload_status {
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
