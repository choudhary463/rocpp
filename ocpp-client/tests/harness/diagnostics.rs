use std::task::{Context, Poll};

use chrono::{DateTime, Utc};
use rocpp_client::v16::{Diagnostics, DiagnosticsResponse};

pub struct MockDiagnostics {
    file_name: Option<String>,
    res: Option<DiagnosticsResponse>,
}

impl MockDiagnostics {
    pub fn new() -> Self {
        Self {
            file_name: None,
            res: None,
        }
    }
}

impl Diagnostics for MockDiagnostics {
    async fn get_file_name(
        &mut self,
        _start_time: Option<DateTime<Utc>>,
        _stop_time: Option<DateTime<Utc>>,
    ) -> Option<String> {
        assert!(self.file_name.is_none());
        assert!(self.res.is_none());
        let file_name = "hello.txt".to_string();
        self.file_name = Some(file_name.clone());
        Some(file_name)
    }
    async fn diagnostics_upload(&mut self, location: String, timeout: u64) {
        let file_name = self.file_name.clone().unwrap();
        log::info!(
            "uploading at location: {}, file_name: {}, timeout: {}",
            location,
            file_name,
            timeout
        );
        let res = location == format!("valid_location");
        let res = res
            .then(|| DiagnosticsResponse::Success)
            .unwrap_or(DiagnosticsResponse::Failed);
        self.res = Some(res)
    }
    fn poll_diagnostics_upload(&mut self, _cx: &mut Context<'_>) -> Poll<DiagnosticsResponse> {
        if let Some(res) = self.res.take() {
            self.file_name.take();
            Poll::Ready(res)
        } else {
            Poll::Pending
        }
    }
}
