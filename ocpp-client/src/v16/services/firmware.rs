use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use alloc::{boxed::Box, string::String, vec::Vec};

use crate::v16::interface::Firmware;

pub(crate) enum FirmwareStage<F: Firmware> {
    Idle(F),
    Downloading(Pin<Box<dyn Future<Output = (F, Option<Vec<u8>>)> + Send>>),
    Installing(Pin<Box<dyn Future<Output = (F, bool)> + Send>>),
    Empty,
}

#[derive(Debug)]
pub enum FirmwareResponse {
    DownloadStatus(Option<Vec<u8>>),
    InstallStatus(bool),
}

pub(crate) struct FirmwareService<F: Firmware> {
    state: FirmwareStage<F>,
}

impl<F: Firmware> FirmwareService<F> {
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

impl<F: Firmware> Future for FirmwareService<F> {
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
