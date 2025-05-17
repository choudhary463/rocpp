use std::{
    collections::VecDeque,
    future::poll_fn,
    sync::{Arc, Mutex},
    task::Poll,
    time::{Duration, Instant},
};

use futures::task::AtomicWaker;
use ocpp_core::v16::protocol_error::ProtocolError;
use serde_json::Value;
use tokio::time;

#[derive(Clone, Debug)]
pub enum ConnectionEvents {
    Connected(String),
    WsMsg(Option<String>, Result<Value, ProtocolError>),
    Disconnected,
    Invalid,
    Timeout,
}

#[derive(Clone, Debug)]
pub enum SeccEvents {
    HardReset,
    Crashed,
}

#[derive(Clone, Debug)]
pub enum Event {
    Connection(ConnectionEvents),
    Secc(SeccEvents),
}

#[derive(Debug)]
struct Inner {
    list: Mutex<VecDeque<Event>>,
    waker: AtomicWaker,
}

impl Inner {
    fn push(&self, ev: Event) {
        self.list.lock().unwrap().push_back(ev);
        self.waker.wake();
    }
}

#[derive(Clone, Debug)]
pub struct EventTx {
    inner: Arc<Inner>,
}

impl EventTx {
    pub fn push(&self, ev: Event) {
        self.inner.push(ev);
    }
}

#[derive(Debug)]
pub struct EventRx {
    inner: Arc<Inner>,
    timeout: Duration,
}

impl EventRx {
    pub async fn next(&self) -> Option<(Event, Duration)> {
        let start = Instant::now();

        let wait_one = poll_fn(|cx| {
            if let Some(ev) = self.inner.list.lock().unwrap().pop_front() {
                return Poll::Ready(ev);
            }
            self.inner.waker.register(cx.waker());
            if let Some(ev) = self.inner.list.lock().unwrap().pop_front() {
                Poll::Ready(ev)
            } else {
                Poll::Pending
            }
        });

        match time::timeout(self.timeout, wait_one).await {
            Ok(ev) => Some((ev, start.elapsed())),
            Err(_) => None,
        }
    }
}

pub fn event_bus(timeout: u64) -> (EventTx, EventRx) {
    let inner = Arc::new(Inner {
        list: Default::default(),
        waker: AtomicWaker::new(),
    });
    (
        EventTx {
            inner: inner.clone(),
        },
        EventRx {
            inner,
            timeout: Duration::from_secs(timeout),
        },
    )
}
