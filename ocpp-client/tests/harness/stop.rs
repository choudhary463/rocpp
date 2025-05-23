use std::{future::Future, pin::Pin, task::Poll};

use futures::FutureExt;
use ocpp_client::v16::StopChargePoint;
use tokio_util::sync::CancellationToken;

pub struct StopService {
    token: CancellationToken,
    fut: Option<Pin<Box<dyn Future<Output = ()> + Send>>>
}

impl StopService {
    pub fn new() -> Self {
        let token = CancellationToken::new();
        Self {
            token: CancellationToken::new(),
            fut: Some(Box::pin(async move { token.cancelled().await; }))
        }
    }
    pub fn get_token(&self) -> CancellationToken {
        self.token.clone()
    }
}

impl StopChargePoint for StopService {
    fn poll_stopped(&mut self,cx: &mut std::task::Context<'_>) -> std::task::Poll<()> {
        if let Some(fut) = self.fut.as_mut() {
            match fut.poll_unpin(cx) {
                Poll::Ready(_) => {
                    self.fut.take();
                    Poll::Ready(())
                },
                Poll::Pending => Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}