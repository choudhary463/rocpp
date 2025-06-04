use core::{task::{Context, Poll}, time::Duration};

use flume::{r#async::RecvFut, unbounded, Receiver, Sender};
use futures::{FutureExt, SinkExt, StreamExt};
use rocpp_client::v16::{Websocket, WsEvent};
use tokio_tungstenite::{connect_async, tungstenite::{client::IntoClientRequest, http::HeaderValue, Message}};
use tokio_util::sync::CancellationToken;


pub struct WsClient {
    ev_tx: Sender<WsEvent>,
    ev_rx: Receiver<WsEvent>,
    ev_rx_fut: RecvFut<'static, WsEvent>,
    msg_tx: Sender<Option<String>>,
    msg_rx: Receiver<Option<String>>
}

impl WsClient {
    pub fn new() -> Self {
        let (ev_tx, ev_rx) = unbounded();
        let (msg_tx, msg_rx) = unbounded();
        Self {
            ev_tx,
            ev_rx: ev_rx.clone(),
            ev_rx_fut: ev_rx.into_recv_async(),
            msg_tx,
            msg_rx
        }
    }
}

impl Websocket for WsClient {
    async fn ws_connect(&mut self, url: String) {
        let _ = self.ev_rx.drain();
        let _ = self.msg_rx.drain();
        let ev_tx = self.ev_tx.clone();
        let msg_rx = self.msg_rx.clone();
        tokio::spawn(ws_task(url, ev_tx, msg_rx));
    }
    async fn ws_send(&mut self, msg: String) {
        self.msg_tx.send_async(Some(msg)).await.unwrap();
    }
    async fn ws_close(&mut self) {
        self.msg_tx.send_async(None).await.unwrap();
    }
    fn poll_ws_recv(&mut self, cx: &mut Context<'_>) -> Poll<WsEvent> {
        match self.ev_rx_fut.poll_unpin(cx) {
            Poll::Ready(t) => Poll::Ready(t.unwrap()),
            Poll::Pending => Poll::Pending
        }
    }
}

async fn ws_task(url: String, ev_tx: Sender<WsEvent>, msg_rx: Receiver<Option<String>>) {
    let mut req = url.into_client_request().unwrap();
    let headers = req.headers_mut();
    headers.insert(
        tokio_tungstenite::tungstenite::http::header::SEC_WEBSOCKET_PROTOCOL,
        HeaderValue::from_str("ocpp1.6").unwrap(),
    );
    let stream = loop {
        match connect_async(req.clone()).await {
            Ok(t) => {
                break t.0;
            }
            Err(e) => {
                log::error!("ws error:{:?}", e);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    };
    ev_tx.send_async(WsEvent::Connected).await.unwrap();
    let (mut ws_tx, mut ws_rx) = stream.split();
    let token = CancellationToken::new();
    let stop_token = token.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = stop_token.cancelled() => {
                    break;
                }
                msg = msg_rx.recv_async() => {
                    let msg = msg.unwrap();
                    let msg = match msg {
                        Some(t) => Message::Text(t.into()),
                        None => Message::Close(None)
                    };
                    match ws_tx.send(msg).await {
                        Ok(_) => {},
                        Err(_) => {
                            break;
                        },
                    }
                }
            }
        }
    });
    loop {
        let req = ws_rx.next().await;
        if let Some(req) = req {
            match req {
                Ok(t) => {
                    match t {
                        Message::Text(t) => {
                            ev_tx.send_async(WsEvent::Msg(t.to_string())).await.unwrap();
                        }
                        Message::Ping(_) => {

                        }
                        _ => {
                            break;
                        }
                    }
                }
                Err(_) => {
                    break;
                }
            }
        } else {
            break;
        }
    }
    token.cancel();
    ev_tx.send_async(WsEvent::Disconnected).await.unwrap();
}