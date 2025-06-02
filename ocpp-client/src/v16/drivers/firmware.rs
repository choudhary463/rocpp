#[cfg(feature = "async")]
use {core::{future::Future, pin::Pin, task::{Context, Poll}}, alloc::{boxed::Box, string::String, vec::Vec}};

#[cfg(feature = "ftp_transfer")]
use super::ftp::FtpService;

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait FirmwareDownload: Send + Unpin + 'static {
    async fn download(&mut self, location: String) -> Option<Vec<u8>>;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait FirmwareInstall: Send + Unpin + 'static {
    async fn install(&mut self, firmware_image: Vec<u8>) -> bool;
}

#[cfg(feature = "async")]
pub trait Firmware: FirmwareDownload + FirmwareInstall {}

#[cfg(feature = "async")]
enum FirmwareStage<F: Firmware> {
    Idle(F),
    Downloading(Pin<Box<dyn Future<Output = (F, Option<Vec<u8>>)> + Send>>),
    Installing(Pin<Box<dyn Future<Output = (F, bool)> + Send>>),
    Empty,
}

#[cfg(feature = "async")]
#[derive(Debug)]
pub(crate) enum FirmwareResponse {
    DownloadStatus(Option<Vec<u8>>),
    InstallStatus(bool),
}

#[cfg(feature = "async")]
pub(crate) struct FirmwareManager<F: Firmware> {
    state: FirmwareStage<F>,
}

#[cfg(feature = "async")]
impl<F: Firmware> FirmwareManager<F> {
    pub fn new(fw: F) -> Self {
        Self {
            state: FirmwareStage::Idle(fw),
        }
    }
    pub async fn make_idle(&mut self) {
        match core::mem::replace(&mut self.state, FirmwareStage::Empty) {
            FirmwareStage::Idle(t) => {
                self.state = FirmwareStage::Idle(t);
            }
            FirmwareStage::Downloading(mut t) => {
                let res = t.as_mut().await.0;
                self.state = FirmwareStage::Idle(res);
            }
            FirmwareStage::Installing(mut t) => {
                let res = t.as_mut().await.0;
                self.state = FirmwareStage::Idle(res);
            }
            _ => {
                unreachable!();
            }
        }
    }
    pub fn start_firmware_download(&mut self, location: String) {
        let mut f = match core::mem::replace(&mut self.state, FirmwareStage::Empty) {
            FirmwareStage::Idle(d) => d,
            _ => {
                unreachable!();
            }
        };
        let download_fut = async move {
            let res = f.download(location).await;
            (f, res)
        };
        self.state = FirmwareStage::Downloading(Box::pin(download_fut));
    }

    pub fn start_firmware_install(&mut self, artifact: Vec<u8>) {
        let mut f = match core::mem::replace(&mut self.state, FirmwareStage::Empty) {
            FirmwareStage::Idle(d) => d,
            _ => {
                unreachable!();
            }
        };
        let install_fut = async move {
            let res = f.install(artifact).await;
            (f, res)
        };
        self.state = FirmwareStage::Installing(Box::pin(install_fut));
    }
}

#[cfg(feature = "async")]
impl<F: Firmware> Future for FirmwareManager<F> {
    type Output = FirmwareResponse;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.state {
            FirmwareStage::Idle(_) => Poll::Pending,
            FirmwareStage::Downloading(fut) => match fut.as_mut().poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready((a, b)) => {
                    self.state = FirmwareStage::Idle(a);
                    Poll::Ready(FirmwareResponse::DownloadStatus(b))
                }
            },
            FirmwareStage::Installing(fut) => match fut.as_mut().poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready((a, b)) => {
                    self.state = FirmwareStage::Idle(a);
                    Poll::Ready(FirmwareResponse::InstallStatus(b))
                }
            },
            FirmwareStage::Empty => {
                unreachable!();
            }
        }
    }
}

#[cfg(feature = "ftp_transfer")]
pub struct FtpFirmwareDownload {}

#[cfg(feature = "ftp_transfer")]
impl FtpFirmwareDownload {
    pub fn new() -> Self {
        Self {  }
    }
}

#[cfg(feature = "ftp_transfer")]
#[async_trait::async_trait]
impl FirmwareDownload for FtpFirmwareDownload {
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
}