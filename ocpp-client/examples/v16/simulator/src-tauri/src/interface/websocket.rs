use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use async_trait::async_trait;
use futures_util::{stream::StreamExt, Sink, SinkExt};
use ocpp_client::v16::WebsocketIo;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, Message},
    MaybeTlsStream, WebSocketStream,
};

pub struct WsService {
    socket: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl WsService {
    pub fn new() -> Self {
        Self { socket: None }
    }
}

#[async_trait]
impl WebsocketIo for WsService {
    async fn connect(&mut self, url: String) {
        let mut req = url.into_client_request().unwrap();
        let headers = req.headers_mut();
        headers.insert(
            tokio_tungstenite::tungstenite::http::header::SEC_WEBSOCKET_PROTOCOL,
            HeaderValue::from_str("ocpp1.6").unwrap(),
        );
        loop {
            match connect_async(req.clone()).await {
                Ok(t) => {
                    self.socket = Some(t.0);
                    return;
                }
                Err(e) => {
                    log::error!("ws error:{:?}", e);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<String>> {
        let socket = match self.socket.as_mut() {
            Some(sock) => sock,
            None => return Poll::Ready(None),
        };
        loop {
            match socket.poll_next_unpin(cx) {
                Poll::Ready(Some(Ok(Message::Ping(payload)))) => {
                    let mut sink = Pin::new(&mut *socket);
                    if let Poll::Ready(Ok(())) = sink.as_mut().poll_ready(cx) {
                        let _ = sink.as_mut().start_send(Message::Pong(payload));
                        let _ = sink.as_mut().poll_flush(cx);
                    }
                    continue;
                }
                Poll::Ready(Some(Ok(Message::Text(s)))) => return Poll::Ready(Some(s.to_string())),
                Poll::Ready(Some(Ok(_))) => {
                    self.socket = None;
                    return Poll::Ready(None);
                }
                Poll::Ready(Some(Err(_))) | Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }

    async fn send(&mut self, msg: String) {
        if let Some(socket) = &mut self.socket {
            let _ = socket.send(Message::Text(msg.into())).await;
        }
    }

    async fn close(&mut self) {
        if let Some(mut socket) = self.socket.take() {
            let _ = socket.close(None).await;
        }
    }
}
