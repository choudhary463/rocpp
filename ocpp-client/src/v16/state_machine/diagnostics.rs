use alloc::string::String;
use chrono::{DateTime, Utc};
use ocpp_core::v16::{
    messages::diagnostics_status_notification::DiagnosticsStatusNotificationRequest,
    types::DiagnosticsStatus,
};

use crate::v16::{
    interface::{Database, Secc},
    cp::ChargePointCore
};

use super::call::CallAction;

pub(crate) struct DiagnosticsUploadInfo {
    pub retry_left: u64,
    pub retry_interval: u64,
    pub location: String,
    pub file_name: String,
    pub start_time: Option<DateTime<Utc>>,
    pub stop_time: Option<DateTime<Utc>>,
}

pub(crate) enum DiagnosticsState {
    Idle,
    Uploading(DiagnosticsUploadInfo),
}

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub(crate) fn send_diagnostics_status_notification(&mut self, status: DiagnosticsStatus) {
        let payload = DiagnosticsStatusNotificationRequest { status };
        self.enqueue_call(CallAction::DiagnosticsStatusNotification, payload);
    }
    pub(crate) fn try_diagnostrics_upload(&mut self) {
        if let DiagnosticsState::Uploading(t) =
            core::mem::replace(&mut self.diagnostics_state, DiagnosticsState::Idle)
        {
            if t.retry_left == 0 {
                self.send_diagnostics_status_notification(DiagnosticsStatus::UploadFailed);
            } else {
                self.start_diagnostics_upload(
                    t.location.clone(),
                    t.file_name.clone(),
                    t.start_time,
                    t.stop_time,
                    t.retry_interval
                );
                self.diagnostics_state = DiagnosticsState::Uploading(t);
            }
        } else {
            unreachable!();
        }
    }
    pub(crate) fn trigger_diagnostics_status_notification(&mut self) {
        let status = match self.diagnostics_state {
            DiagnosticsState::Idle => DiagnosticsStatus::Idle,
            DiagnosticsState::Uploading { .. } => DiagnosticsStatus::Uploading,
        };
        self.send_diagnostics_status_notification(status);
    }
}
