#[cfg(feature = "async")]
use {core::{future::Future, pin::Pin, task::{Context, Poll}}, alloc::{boxed::Box, string::String}, chrono::{DateTime, Utc}};

#[cfg(feature = "ftp_transfer")]
use {super::ftp::FtpService, std::time::Duration};

#[derive(Debug)]
pub enum DiagnosticsResponse {
    Timeout,
    Success,
    Failed
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait Diagnostics: Send + Unpin + 'static {
    async fn upload(
        &mut self,
        location: String,
        file_name: String,
        start_time: Option<DateTime<Utc>>,
        stop_time: Option<DateTime<Utc>>,
        timeout: u64
    ) -> DiagnosticsResponse;
}

#[cfg(feature = "async")]
enum DiagnosticsStage<D: Diagnostics> {
    Idle(D),
    Uploading(Pin<Box<dyn Future<Output = (D, DiagnosticsResponse)> + Send>>),
    Empty,
}

#[cfg(feature = "async")]
pub(crate) struct DiagnosticsManager<D: Diagnostics> {
    state: DiagnosticsStage<D>
}


#[cfg(feature = "async")]
impl<D: Diagnostics> DiagnosticsManager<D> {
    pub fn new(di: D) -> Self {
        Self {
            state: DiagnosticsStage::Idle(di)
        }
    }
    pub async fn make_idle(&mut self) {
        match core::mem::replace(&mut self.state, DiagnosticsStage::Empty) {
            DiagnosticsStage::Idle(t) => {
                self.state = DiagnosticsStage::Idle(t);
            }
            DiagnosticsStage::Uploading(mut t) => {
                let res = t.as_mut().await.0;
                self.state = DiagnosticsStage::Idle(res);
            }
            _ => {
                unreachable!();
            }
        }
    }
    pub fn start_upload(
        &mut self,
        location: String,
        file_name: String,
        start_time: Option<DateTime<Utc>>,
        stop_time: Option<DateTime<Utc>>,
        timeout: u64
    ) {
        let mut d = match core::mem::replace(&mut self.state, DiagnosticsStage::Empty) {
            DiagnosticsStage::Idle(d) => d,
            _ => {
                unreachable!();
            }
        };
        let upload_fut = async move {
            let res = d.upload(location, file_name, start_time, stop_time, timeout).await;
            (d, res)
        };
        self.state = DiagnosticsStage::Uploading(Box::pin(upload_fut));
    }
}

#[cfg(feature = "async")]
impl<D: Diagnostics> Future for DiagnosticsManager<D> {
    type Output = DiagnosticsResponse;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.state {
            DiagnosticsStage::Idle(_) => Poll::Pending,
            DiagnosticsStage::Uploading(fut) => match fut.as_mut().poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready((a, b)) => {
                    self.state = DiagnosticsStage::Idle(a);
                    Poll::Ready(b)
                }
            },
            DiagnosticsStage::Empty => {
                unreachable!();
            }
        }
    }
}


#[cfg(feature = "ftp_transfer")]
pub struct FtpDiagnostics {}

#[cfg(feature = "ftp_transfer")]
impl FtpDiagnostics {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(feature = "ftp_transfer")]
#[async_trait::async_trait]
impl Diagnostics for FtpDiagnostics {
    async fn upload(
        &mut self,
        location: String,
        file_name: String,
        start_time: Option<DateTime<Utc>>,
        stop_time: Option<DateTime<Utc>>,
        mut timeout: u64
    ) -> DiagnosticsResponse {
        let ftp = FtpService::new(location);
        if timeout == 0 {
            timeout = 100;
        }
        match tokio::time::timeout(Duration::from_secs(timeout), ftp.upload(file_name, start_time, stop_time)).await {
            Ok(t) => {
                match t {
                    Ok(_) => {
                        DiagnosticsResponse::Success
                    }
                    Err(e) => {
                        log::error!("upload error {:?}", e);
                        DiagnosticsResponse::Failed
                    }
                }
            },
            Err(_) => {
                log::error!("upload timeout");
                DiagnosticsResponse::Timeout
            }
        }
    }
}