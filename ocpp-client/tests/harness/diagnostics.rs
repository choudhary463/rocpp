use chrono::{DateTime, Utc};
use ocpp_client::v16::Diagnostics;

pub struct MockDiagnostics {}

#[async_trait::async_trait]
impl Diagnostics for MockDiagnostics {
    async fn upload(
        &mut self,
        location: String,
        file_name: String,
        _start_time: Option<DateTime<Utc>>,
        _stop_time: Option<DateTime<Utc>>,
    ) -> bool {
        log::info!(
            "uploading at location: {}, file name: {}",
            location,
            file_name
        );
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        return location == format!("valid_location");
    }
}
