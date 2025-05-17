use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::v16::interface::Firmware;

pub(crate) enum FirmwareState<F: Firmware> {
    Idle(F),
    Downloading(Pin<Box<dyn Future<Output = (F, Option<Vec<u8>>)> + Send>>),
    Installing(Pin<Box<dyn Future<Output = (F, bool)> + Send>>),
    Empty,
}

#[derive(Debug)]
pub(crate) enum FirmwareResponse {
    DownloadStatus(Option<Vec<u8>>),
    InstallStatus(bool),
}

pub(crate) struct FirmwareService<F: Firmware> {
    state: FirmwareState<F>,
}

impl<F: Firmware> FirmwareService<F> {
    pub fn new(fw: F) -> Self {
        Self {
            state: FirmwareState::Idle(fw),
        }
    }
    pub async fn make_idle(&mut self) {
        match std::mem::replace(&mut self.state, FirmwareState::Empty) {
            FirmwareState::Idle(t) => {
                self.state = FirmwareState::Idle(t);
            }
            FirmwareState::Downloading(mut t) => {
                let res = t.as_mut().await.0;
                self.state = FirmwareState::Idle(res);
            }
            FirmwareState::Installing(mut t) => {
                let res = t.as_mut().await.0;
                self.state = FirmwareState::Idle(res);
            }
            _ => {
                unreachable!();
            }
        }
    }
    pub fn start_firmware_download(&mut self, location: String) {
        let mut f = match std::mem::replace(&mut self.state, FirmwareState::Empty) {
            FirmwareState::Idle(d) => d,
            _ => {
                unreachable!();
            }
        };
        let download_fut = async move {
            let res = f.download(location).await;
            (f, res)
        };
        self.state = FirmwareState::Downloading(Box::pin(download_fut));
    }

    pub fn start_firmware_install(&mut self, artifact: Vec<u8>) {
        let mut f = match std::mem::replace(&mut self.state, FirmwareState::Empty) {
            FirmwareState::Idle(d) => d,
            _ => {
                unreachable!();
            }
        };
        let install_fut = async move {
            let res = f.install(artifact).await;
            (f, res)
        };
        self.state = FirmwareState::Installing(Box::pin(install_fut));
    }
}

impl<F: Firmware> Future for FirmwareService<F> {
    type Output = FirmwareResponse;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.state {
            FirmwareState::Idle(_) => Poll::Pending,
            FirmwareState::Downloading(fut) => match fut.as_mut().poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready((a, b)) => {
                    self.state = FirmwareState::Idle(a);
                    Poll::Ready(FirmwareResponse::DownloadStatus(b))
                }
            },
            FirmwareState::Installing(fut) => match fut.as_mut().poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready((a, b)) => {
                    self.state = FirmwareState::Idle(a);
                    Poll::Ready(FirmwareResponse::InstallStatus(b))
                }
            },
            FirmwareState::Empty => {
                unreachable!();
            }
        }
    }
}
