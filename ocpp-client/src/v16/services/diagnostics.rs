use core::{future::Future, pin::Pin, task::{Context, Poll}};

use alloc::{boxed::Box, string::String};
use chrono::{DateTime, Utc};

use crate::v16::{interface::Diagnostics, DiagnosticsResponse};

pub(crate) enum DiagnosticsStage<D: Diagnostics> {
    Idle(D),
    Uploading(Pin<Box<dyn Future<Output = (D, DiagnosticsResponse)> + Send>>),
    Empty,
}

pub(crate) struct DiagnosticsService<D: Diagnostics> {
    state: DiagnosticsStage<D>
}

impl<D: Diagnostics> DiagnosticsService<D> {
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

impl<D: Diagnostics> Future for DiagnosticsService<D> {
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


