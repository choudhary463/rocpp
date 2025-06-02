use async_trait::async_trait;

use ocpp_client::v16::{Firmware, FirmwareDownload, FirmwareInstall, FtpFirmwareDownload};
use regex::Regex;

use super::database::DatabaseService;

pub struct FirmwareService {
    db: DatabaseService,
    download: FtpFirmwareDownload,
}

impl FirmwareService {
    pub fn new(db: DatabaseService, download: FtpFirmwareDownload) -> Self {
        Self { db, download }
    }
}

#[async_trait]
impl FirmwareDownload for FirmwareService {
    async fn download(&mut self, location: String) -> Option<Vec<u8>> {
        self.download.download(location).await
    }
}

#[async_trait]
impl FirmwareInstall for FirmwareService {
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


#[async_trait]
impl Firmware for FirmwareService { }
