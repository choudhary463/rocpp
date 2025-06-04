use alloc::string::String;
use rocpp_core::v16::{
    messages::diagnostics_status_notification::DiagnosticsStatusNotificationRequest,
    types::DiagnosticsStatus,
};

use crate::v16::{cp::ChargePoint, interfaces::ChargePointInterface};

use super::call::CallAction;

pub(crate) struct DiagnosticsUploadInfo {
    pub retry_left: u64,
    pub retry_interval: u64,
    pub location: String
}

pub(crate) enum DiagnosticsState {
    Idle,
    Uploading(DiagnosticsUploadInfo),
}

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn send_diagnostics_status_notification(&mut self, status: DiagnosticsStatus) {
        let payload = DiagnosticsStatusNotificationRequest { status };
        self.enqueue_call(CallAction::DiagnosticsStatusNotification, payload).await;
    }
    pub(crate) async fn try_diagnostrics_upload(&mut self) {
        if let DiagnosticsState::Uploading(t) =
            core::mem::replace(&mut self.diagnostics_state, DiagnosticsState::Idle)
        {
            if t.retry_left == 0 {
                self.send_diagnostics_status_notification(DiagnosticsStatus::UploadFailed).await;
            } else {
                self.start_diagnostics_upload(
                    t.location.clone(),
                    t.retry_interval
                ).await;
                self.diagnostics_state = DiagnosticsState::Uploading(t);
            }
        } else {
            unreachable!();
        }
    }
    pub(crate) async fn trigger_diagnostics_status_notification(&mut self) {
        let status = match self.diagnostics_state {
            DiagnosticsState::Idle => DiagnosticsStatus::Idle,
            DiagnosticsState::Uploading { .. } => DiagnosticsStatus::Uploading,
        };
        self.send_diagnostics_status_notification(status).await;
    }
}
