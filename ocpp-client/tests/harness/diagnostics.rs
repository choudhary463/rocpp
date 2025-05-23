use std::time::Duration;

use chrono::{DateTime, Utc};
use ocpp_client::v16::{Diagnostics, DiagnosticsResponse};

pub struct MockDiagnostics {}

impl MockDiagnostics {
    async fn upload_taks(&self) {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

#[async_trait::async_trait]
impl Diagnostics for MockDiagnostics {
    async fn upload(
        &mut self,
        location: String,
        file_name: String,
        _start_time: Option<DateTime<Utc>>,
        _stop_time: Option<DateTime<Utc>>,
        mut timeout: u64
    ) -> DiagnosticsResponse {
        log::info!(
            "uploading at location: {}, file name: {}",
            location,
            file_name
        );
        if timeout == 0 {
            timeout = 100;
        }
        match tokio::time::timeout(Duration::from_secs(timeout), self.upload_taks()).await {
            Ok(_) => {
                if location == format!("valid_location") {
                    DiagnosticsResponse::Success
                } else {
                    DiagnosticsResponse::Failed
                }
            }
            Err(_) => {
                DiagnosticsResponse::Timeout
            }
        }
    }
}
