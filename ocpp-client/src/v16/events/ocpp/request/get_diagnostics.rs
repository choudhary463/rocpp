use ocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::get_diagnostics::{GetDiagnosticsRequest, GetDiagnosticsResponse},
        types::DiagnosticsStatus,
    },
};

use crate::v16::{
    interface::{Database, Secc},
    state_machine::{
        core::ChargePointCore,
        diagnostics::{DiagnosticsState, DiagnosticsUploadInfo},
    },
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    fn get_diagnostics_file_name(&self) -> Option<String> {
        let s = uuid::Uuid::new_v4().simple().to_string();
        Some(format!("file_{}", &s[..6]))
    }
    pub fn get_diagnostics_ocpp(&mut self, unique_id: String, req: GetDiagnosticsRequest) {
        let (file_name, new_upload) = match &self.diagnostics_state {
            DiagnosticsState::Idle => {
                let res = self.get_diagnostics_file_name();
                let retry_left = req.retries.map(|t| t + 1).unwrap_or(1);
                let retry_interval = req.retry_interval.unwrap_or(0);
                let file_name = res.clone();
                let new_upload = res.map(|f| DiagnosticsUploadInfo {
                    retry_left,
                    retry_interval,
                    location: req.location,
                    file_name: f,
                    start_time: req.start_time,
                    stop_time: req.stop_time,
                });
                (file_name, new_upload)
            }
            DiagnosticsState::Uploading(t) => (Some(t.file_name.clone()), None),
        };
        let payload = GetDiagnosticsResponse {
            file_name: file_name.clone(),
        };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode());

        if let Some(upload) = new_upload {
            self.send_diagnostics_status_notification(DiagnosticsStatus::Uploading);
            self.diagnostics_state = DiagnosticsState::Uploading(upload);
            self.try_diagnostrics_upload();
        }
    }
}
