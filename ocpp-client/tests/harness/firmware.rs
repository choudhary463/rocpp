use std::time::Duration;

use ocpp_client::v16::Firmware;

pub struct MockFirmware {}

#[async_trait::async_trait]
impl Firmware for MockFirmware {
    async fn download(&mut self, location: String) -> Option<Vec<u8>> {
        log::info!("firmware download location: {}", location);
        if location == format!("download_success:install:success") {
            tokio::time::sleep(Duration::from_secs(2)).await;
            return Some(Vec::from(vec![0, 1, 2, 3]));
        } else if location == format!("download_success:install:fail") {
            tokio::time::sleep(Duration::from_secs(2)).await;
            return Some(Vec::new());
        } else {
            return None;
        }
    }
    async fn install(&mut self, artifact: Vec<u8>) -> bool {
        log::info!("firmware installing artifact: {:?}", artifact);
        tokio::time::sleep(Duration::from_secs(5)).await;
        return !artifact.is_empty();
    }
}
