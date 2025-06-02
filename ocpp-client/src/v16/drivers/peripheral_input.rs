#[cfg(feature = "async")]
use core::{future::Future, pin::Pin, task::{Context, Poll}};

use alloc::string::String;
use ocpp_core::v16::types::ChargePointErrorCode;

#[cfg(feature = "flume_peripheral")]
use {flume::{unbounded, Receiver, Sender}, futures::FutureExt, alloc::boxed::Box};

#[derive(Clone, PartialEq, Debug)]
pub enum SeccState {
    Plugged,
    Unplugged,
    Faulty,
}

#[derive(Debug)]
pub enum PeripheralActions {
    State(
        usize,
        SeccState,
        Option<ChargePointErrorCode>,
        Option<String>,
    ),
    IdTag(usize, String),
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait PeripheralInput: Send + Unpin + 'static {
    fn poll_next_input(&mut self, cx: &mut Context<'_>) -> Poll<PeripheralActions>;
}

#[cfg(feature = "async")]
pub(crate) struct PeripheralDriver<T: PeripheralInput> {
    peripheral: T
}

#[cfg(feature = "async")]
impl<T: PeripheralInput> PeripheralDriver<T> {
    pub fn new(peripheral: T) -> Self {
        Self {
            peripheral
        }
    }
}

#[cfg(feature = "async")]
impl<T: PeripheralInput> Future for PeripheralDriver<T> {
    type Output = PeripheralActions;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.peripheral.poll_next_input(cx)
    }
}

#[cfg(feature = "flume_peripheral")]
pub struct FlumePeripheral {
    tx: Sender<PeripheralActions>,
    fut: Pin<alloc::boxed::Box<dyn Future<Output = (PeripheralActions, Receiver<PeripheralActions>)> + Send>>
}

#[cfg(feature = "flume_peripheral")]
impl FlumePeripheral {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self {
            tx,
            fut: alloc::boxed::Box::pin(async move { let res = rx.recv_async().await.unwrap(); (res, rx) })
        }
    }
    pub fn from_channel(tx: Sender<PeripheralActions>, rx: Receiver<PeripheralActions>) -> Self {
        Self {
            tx,
            fut: alloc::boxed::Box::pin(async move { let res = rx.recv_async().await.unwrap(); (res, rx) })
        }
    }
    pub fn get_sender(&self) -> Sender<PeripheralActions> {
        self.tx.clone()
    }
}

#[cfg(feature = "flume_peripheral")]
impl PeripheralInput for FlumePeripheral {
    fn poll_next_input(&mut self, cx: &mut core::task::Context<'_>) -> core::task::Poll<PeripheralActions> {
        match self.fut.poll_unpin(cx) {
            Poll::Ready((action, rx)) => {
                self.fut = Box::pin(async move { let res = rx.recv_async().await.unwrap(); (res, rx) });
                Poll::Ready(action)
            }
            Poll::Pending => {
                Poll::Pending
            }
        }
    }
}