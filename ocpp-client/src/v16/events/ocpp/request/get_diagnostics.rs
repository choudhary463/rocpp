use alloc::string::String;
use rocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::get_diagnostics::{GetDiagnosticsRequest, GetDiagnosticsResponse},
        types::DiagnosticsStatus,
    },
};

use crate::v16::{cp::ChargePoint, interfaces::ChargePointInterface, state_machine::diagnostics::{DiagnosticsState, DiagnosticsUploadInfo}};


impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn get_diagnostics_ocpp(&mut self, unique_id: String, req: GetDiagnosticsRequest) {
        let (file_name, new_upload) = match &self.diagnostics_state {
            DiagnosticsState::Idle => {
                match self.interface.interface.get_file_name(req.start_time, req.stop_time).await {
                    Some(t) =>  {
                        let retry_left = req.retries.map(|t| t + 1).unwrap_or(1);
                        let retry_interval = req.retry_interval.unwrap_or(0);
                        let new_upload = DiagnosticsUploadInfo {
                            retry_left,
                            retry_interval,
                            location: req.location
                        };
                        (Some(t), Some(new_upload))
                    },
                    None => {
                        (None, None)
                    }
                }
            }
            DiagnosticsState::Uploading(_) => (None, None),
        };
        let payload = GetDiagnosticsResponse {
            file_name: file_name.clone(),
        };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode()).await;

        if let Some(upload) = new_upload {
            self.send_diagnostics_status_notification(DiagnosticsStatus::Uploading).await;
            self.diagnostics_state = DiagnosticsState::Uploading(upload);
            self.try_diagnostrics_upload().await;
        }
    }
}
