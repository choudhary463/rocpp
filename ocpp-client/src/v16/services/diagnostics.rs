use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use chrono::{DateTime, Utc};
use tokio_util::sync::CancellationToken;

use crate::v16::interface::Diagnostics;

pub(crate) enum DiagnosticsState<D: Diagnostics> {
    Idle(D),
    Uploading(Pin<Box<dyn Future<Output = (D, bool)> + Send>>),
    Empty,
}

pub(crate) struct DiagnosticsService<D: Diagnostics> {
    state: DiagnosticsState<D>,
    cancel_token: Option<CancellationToken>,
}

impl<D: Diagnostics> DiagnosticsService<D> {
    pub fn new(di: D) -> Self {
        Self {
            state: DiagnosticsState::Idle(di),
            cancel_token: None,
        }
    }
    pub async fn make_idle(&mut self) {
        match std::mem::replace(&mut self.state, DiagnosticsState::Empty) {
            DiagnosticsState::Idle(t) => {
                self.state = DiagnosticsState::Idle(t);
            }
            DiagnosticsState::Uploading(mut t) => {
                let res = t.as_mut().await.0;
                self.state = DiagnosticsState::Idle(res);
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
    ) {
        let mut d = match std::mem::replace(&mut self.state, DiagnosticsState::Empty) {
            DiagnosticsState::Idle(d) => d,
            _ => {
                unreachable!();
            }
        };
        let cancel_token = CancellationToken::new();
        let cancel = cancel_token.clone();
        self.cancel_token = Some(cancel_token);
        let upload_fut = async move {
            tokio::select! {
                res = d.upload(location, file_name, start_time, stop_time) => (d, res),
                _ = cancel.cancelled() => (d, false),
            }
        };
        self.state = DiagnosticsState::Uploading(Box::pin(upload_fut));
    }

    pub fn cancel_upload(&mut self) {
        if let Some(token) = self.cancel_token.as_mut() {
            token.cancel();
        }
    }
}

impl<D: Diagnostics> Future for DiagnosticsService<D> {
    type Output = bool;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.state {
            DiagnosticsState::Idle(_) => Poll::Pending,
            DiagnosticsState::Uploading(fut) => match fut.as_mut().poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready((a, b)) => {
                    self.state = DiagnosticsState::Idle(a);
                    self.cancel_token = None;
                    Poll::Ready(b)
                }
            },
            DiagnosticsState::Empty => {
                unreachable!();
            }
        }
    }
}
