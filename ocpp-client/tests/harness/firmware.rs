use std::{task::{Context, Poll}};

use rocpp_client::v16::Firmware;

pub struct MockFirmware {
    download_res: Option<bool>,
    is_downloaded: Option<bool>,
    install_res: Option<bool>
}

impl MockFirmware {
    pub fn new() -> Self {
        Self {
            download_res: None,
            is_downloaded: None,
            install_res: None
        }
    }
}

impl Firmware for MockFirmware {
    async fn firmware_download(&mut self, location: String) {
        log::info!("firmware download location: {}", location);
        let download = if location == format!("download_success:install:success") {
            self.is_downloaded = Some(true);
            true
        } else if location == format!("download_success:install:fail") {
            self.is_downloaded = Some(false);
            true
        } else {
            false
        };
        self.download_res = Some(download);
    }
    async fn firmware_install(&mut self) {
        assert!(self.is_downloaded.is_some());
        self.install_res = self.is_downloaded.take();
        assert!(self.install_res.is_some());
    }
    fn poll_firmware_download(&mut self, _cx: &mut Context<'_>) -> Poll<bool> {
        if let Some(res) = self.download_res.take() {
            Poll::Ready(res)
        } else {
            Poll::Pending
        }
    }
    fn poll_firmware_install(&mut self, _cx: &mut Context<'_>) -> Poll<bool> {
        if let Some(res) = self.install_res.take() {
            Poll::Ready(res)
        } else {
            Poll::Pending
        }
    }
}