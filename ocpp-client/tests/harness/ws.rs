use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    task::{Context, Poll},
};

use futures::task::AtomicWaker;
use ocpp_client::v16::WebsocketIo;
use ocpp_core::{
    format::{
        frame::{Call, CallError, CallResult},
        message::{CallResponse, EncodeDecode, OcppMessage},
    },
    v16::protocol_error::ProtocolError,
};
use tokio::sync::Notify;

use super::event::{ConnectionEvents, Event, EventTx};

#[derive(Debug)]
struct Inner {
    is_connected: AtomicBool,
    inbox: Mutex<VecDeque<String>>,
    event_tx: EventTx,
    waker: AtomicWaker,
    last_call_uid_to_cp: Mutex<Option<String>>,
    last_call_uid_from_cp: Mutex<Option<String>>,
    notify: Mutex<Option<Arc<Notify>>>,
}

pub struct MockWs {
    inner: Arc<Inner>,
}

impl MockWs {
    pub fn new(event_tx: EventTx) -> (Self, MockWsHandle) {
        let inner = Arc::new(Inner {
            is_connected: AtomicBool::new(false),
            inbox: Default::default(),
            event_tx,
            waker: AtomicWaker::new(),
            last_call_uid_to_cp: Default::default(),
            last_call_uid_from_cp: Default::default(),
            notify: Mutex::new(None),
        });
        (
            Self {
                inner: inner.clone(),
            },
            MockWsHandle { inner },
        )
    }
}

#[async_trait::async_trait]
impl WebsocketIo for MockWs {
    async fn connect(&mut self, url: String) {
        log::info!("connecting.......");
        let maybe_notify = { self.inner.notify.lock().unwrap().as_ref().cloned() };
        if let Some(notify) = maybe_notify {
            notify.notified().await;
        }
        log::info!("connected");
        self.inner
            .event_tx
            .push(Event::Connection(ConnectionEvents::Connected(url)));
        self.inner.is_connected.store(true, Ordering::Release);
    }

    async fn send(&mut self, raw: String) {
        let event = match OcppMessage::decode(raw.clone()) {
            OcppMessage::Call(call) => {
                self.inner
                    .last_call_uid_from_cp
                    .lock()
                    .unwrap()
                    .replace(call.unique_id);
                Event::Connection(ConnectionEvents::WsMsg(Some(call.action), Ok(call.payload)))
            }
            OcppMessage::CallResponse(t) => {
                let unique_id = t.get_unique_id();
                match self.inner.last_call_uid_to_cp.lock().unwrap().take() {
                    Some(uid) => {
                        if unique_id == uid {
                            let res = match t {
                                CallResponse::CallResult(t) => Ok(t.payload),
                                CallResponse::CallError(e) => Err(e.error_code),
                            };
                            Event::Connection(ConnectionEvents::WsMsg(None, res))
                        } else {
                            Event::Connection(ConnectionEvents::Invalid)
                        }
                    }
                    None => Event::Connection(ConnectionEvents::Invalid),
                }
            }
            OcppMessage::Invalid(_) => Event::Connection(ConnectionEvents::Invalid),
        };
        self.inner.event_tx.push(event);
    }

    fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<String>> {
        if !self.inner.is_connected.load(Ordering::Acquire) {
            return Poll::Ready(None);
        }
        if let Some(msg) = self.inner.inbox.lock().unwrap().pop_front() {
            return Poll::Ready(Some(msg));
        }
        self.inner.waker.register(cx.waker());
        if !self.inner.is_connected.load(Ordering::Acquire) {
            return Poll::Ready(None);
        }
        if let Some(msg) = self.inner.inbox.lock().unwrap().pop_front() {
            return Poll::Ready(Some(msg));
        }
        Poll::Pending
    }

    async fn close(&mut self) {
        self.inner.is_connected.store(false, Ordering::Release);
        self.inner.inbox.lock().unwrap().clear();
        self.inner
            .event_tx
            .push(Event::Connection(ConnectionEvents::Disconnected));
    }
}

#[derive(Clone, Debug)]
pub struct MockWsHandle {
    inner: Arc<Inner>,
}

impl MockWsHandle {
    pub fn close_connection(&self) {
        assert!(self.inner.is_connected.load(Ordering::Acquire));
        self.inner.is_connected.store(false, Ordering::Release);
        self.inner.inbox.lock().unwrap().clear();
        *self.inner.notify.lock().unwrap() = Some(Arc::new(Notify::new()));
        self.inner
            .event_tx
            .push(Event::Connection(ConnectionEvents::Disconnected));
        self.inner.waker.wake();
    }
    pub fn restore_connection(&self) {
        assert!(!self.inner.is_connected.load(Ordering::Acquire));
        if let Some(notify) = self.inner.notify.lock().unwrap().take() {
            notify.notify_one();
        }
    }
    pub fn inject(&self, msg: String) {
        assert!(self.inner.is_connected.load(Ordering::Acquire));
        self.inner.inbox.lock().unwrap().push_back(msg);
        self.inner.waker.wake();
    }
    pub fn send_call<T: serde::Serialize>(&self, action: &str, payload: T) {
        let unique_id = uuid::Uuid::new_v4().to_string();
        let call = Call {
            unique_id: unique_id.clone(),
            action: action.to_string(),
            payload: serde_json::to_value(payload).unwrap(),
        };
        self.inner
            .last_call_uid_to_cp
            .lock()
            .unwrap()
            .replace(unique_id);
        self.inject(call.encode());
    }
    pub fn send_response<T: serde::Serialize>(&self, payload: Result<T, ProtocolError>) {
        let unique_id = self
            .inner
            .last_call_uid_from_cp
            .lock()
            .unwrap()
            .take()
            .unwrap();
        let res = match payload {
            Ok(t) => CallResponse::CallResult(CallResult::new(unique_id, t)),
            Err(e) => CallResponse::CallError(CallError::new(unique_id, e)),
        };
        self.inject(res.encode());
    }
}
