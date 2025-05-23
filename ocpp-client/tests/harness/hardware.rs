use std::{future::Future, pin::Pin, task::Poll};

use flume::{unbounded, Receiver, Sender};
use futures::FutureExt;
use ocpp_client::v16::{Hardware, HardwareActions};

pub struct HardwareService {
    tx: Sender<HardwareActions>,
    fut: Pin<Box<dyn Future<Output = (HardwareActions, Receiver<HardwareActions>)> + Send>>
}

impl HardwareService {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self {
            tx,
            fut: Box::pin(async move { let res = rx.recv_async().await.unwrap(); (res, rx) })
        }
    }
    
    pub fn get_sender(&self) -> Sender<HardwareActions> {
        self.tx.clone()
    }
}

impl Hardware for HardwareService {
    fn poll_next_action(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<HardwareActions> {
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