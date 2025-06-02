#[cfg(feature = "async")]
use core::{future::Future, pin::Pin, task::{Context, Poll}};

#[cfg(feature = "tokio_shutdown")]
use {tokio_util::sync::CancellationToken, futures::FutureExt};

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait ShutdownSignal: Send + Unpin + 'static {
    fn poll_shutdown(&mut self, cx: &mut Context<'_>) -> Poll<()>;
}

#[cfg(feature = "async")]
pub(crate) struct ShutdownDriver<T: ShutdownSignal> {
    shutdown: T
}

#[cfg(feature = "async")]
impl<T: ShutdownSignal> ShutdownDriver<T> {
    pub fn new(shutdown: T) -> Self {
        Self {
            shutdown
        }
    }
}

#[cfg(feature = "async")]
impl<T: ShutdownSignal> Future for ShutdownDriver<T> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.shutdown.poll_shutdown(cx)
    }
}

#[cfg(feature = "tokio_shutdown")]
pub struct TokioShutdown {
    token: CancellationToken,
    fut: Option<Pin<Box<dyn Future<Output = ()> + Send>>>
}

#[cfg(feature = "tokio_shutdown")]
impl TokioShutdown {
    pub fn new() -> Self {
        let token = CancellationToken::new();
        Self {
            token: CancellationToken::new(),
            fut: Some(Box::pin(async move { token.cancelled().await; }))
        }
    }
    pub fn from_token(token: CancellationToken) -> Self {
        Self {
            token: token.clone(),
            fut: Some(Box::pin(async move { token.cancelled().await; }))
        }
    }
    pub fn get_token(&self) -> CancellationToken {
        self.token.clone()
    }
}

#[cfg(feature = "tokio_shutdown")]
impl ShutdownSignal for TokioShutdown {
    fn poll_shutdown(&mut self,cx: &mut std::task::Context<'_>) -> std::task::Poll<()> {
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