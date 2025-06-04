use std::{future::Future, pin::Pin, task::{Context, Poll}};

use anyhow::Error;
use rocpp_client::v16::Firmware;
use regex::Regex;

use crate::interface::ftp::FtpService;

use super::database::DatabaseService;

pub struct FirmwareService {
    db: DatabaseService,
    downloaded_firmware: Option<Vec<u8>>,
    download_fut: Option<Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>> + Send>>>,
    install_res: Option<bool>
}

impl FirmwareService {
    pub fn new(db: DatabaseService) -> Self {
        Self { db, downloaded_firmware: None, download_fut: None, install_res: None }
    }
}


impl Firmware for FirmwareService { 
    async fn firmware_download(&mut self, location: String) {
        let ftp = FtpService::new(location);
        let fut: Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>> + Send>> = Box::pin(async move { ftp.download().await });
        self.download_fut = Some(fut);
    }
    async fn firmware_install(&mut self) {
        let firmware = self.downloaded_firmware.take().unwrap();
        let res = if let Ok(text) = std::str::from_utf8(&firmware) {
            let version_regex = Regex::new(r"\b\d+\.\d+\.\d+\b").unwrap();
            if let Some(capture) = version_regex.find(text) {
                let version = capture.as_str();
                self.db.set_firmware_version(version.into()).await;
                true
            } else {
                false
            }
        } else {
            false
        };
        self.install_res = Some(res);
    }
    fn poll_firmware_download(&mut self, cx: &mut Context<'_>) -> Poll<bool> {
        let fut = match self.download_fut.as_mut() {
            Some(t) => t,
            None => return Poll::Pending
        };
        match fut.as_mut().poll(cx) {
            Poll::Ready(t) => {
                let res = match t {
                    Ok(t) => {
                        self.downloaded_firmware = Some(t);
                        true
                    }
                    Err(_) => {
                        false
                    }
                };
                self.download_fut.take();
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending
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
