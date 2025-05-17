use async_trait::async_trait;
use chrono::{DateTime, Utc};

use ocpp_client::v16::Diagnostics;

use super::ftp::FtpService;

pub struct DiagnosticsService {}

impl DiagnosticsService {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Diagnostics for DiagnosticsService {
    async fn upload(
        &mut self,
        location: String,
        file_name: String,
        start_time: Option<DateTime<Utc>>,
        stop_time: Option<DateTime<Utc>>,
    ) -> bool {
        let ftp = FtpService::new(location);
        match ftp.upload(file_name, start_time, stop_time).await {
            Ok(_) => true,
            Err(e) => {
                log::error!("upload error {:?}", e);
                false
            }
        }
    }
}
