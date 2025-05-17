use async_trait::async_trait;

use ocpp_client::v16::Firmware;
use regex::Regex;

use super::{database::DatabaseService, ftp::FtpService};

pub struct FirmwareService {
    db: DatabaseService,
}

impl FirmwareService {
    pub fn new(db: DatabaseService) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Firmware for FirmwareService {
    async fn download(&mut self, location: String) -> Option<Vec<u8>> {
        let ftp = FtpService::new(location);
        match ftp.download().await {
            Ok(t) => Some(t),
            Err(e) => {
                log::error!("upload error {:?}", e);
                None
            }
        }
    }
    async fn install(&mut self, firmware_image: Vec<u8>) -> bool {
        if let Ok(text) = std::str::from_utf8(&firmware_image) {
            let version_regex = Regex::new(r"\b\d+\.\d+\.\d+\b").unwrap();
            if let Some(capture) = version_regex.find(text) {
                let version = capture.as_str();
                self.db.set_firmware_version(version.into());
                return true;
            }
        }
        false
    }
}
