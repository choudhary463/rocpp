#[cfg(feature = "async")]
use {core::{future::Future, mem, pin::Pin, task::{Context, Poll}}, alloc::{boxed::Box, string::String}};

#[cfg(feature = "tokio_ws")]
use {tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream, tungstenite::{client::IntoClientRequest, http::HeaderValue, Message}}, tokio::net::TcpStream, std::time::Duration, futures_util::{stream::StreamExt, Sink, SinkExt}};

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait WebsocketTransport: Send + Unpin + 'static {
    async fn connect(&mut self, url: String);
    fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<String>>;
    async fn send(&mut self, msg: String);
    async fn close(&mut self);
}

#[cfg(feature = "async")]
enum WebsocketStage<W: WebsocketTransport> {
    Idle(W),
    Connecting(Pin<Box<dyn Future<Output = W> + Send>>),
    Connected(W),
    Empty,
}

#[cfg(feature = "async")]
#[derive(Debug)]
pub enum WebsocketResponse {
    Connected,
    Disconnected,
    WsMsg(String),
}

#[cfg(feature = "async")]
pub(crate) struct WebsocketClient<W: WebsocketTransport> {
    state: WebsocketStage<W>,
}

#[cfg(feature = "async")]
impl<W: WebsocketTransport> WebsocketClient<W> {
    pub fn new(ws: W) -> Self {
        Self {
            state: WebsocketStage::Idle(ws),
        }
    }

    pub fn connect(&mut self, url: String) {
        let old_state = mem::replace(&mut self.state, WebsocketStage::Empty);

        let ws = match old_state {
            WebsocketStage::Idle(ws) => ws,
            WebsocketStage::Connected(ws) => ws,
            _ => {
                unreachable!();
            }
        };

        let future = async move {
            let mut ws = ws;
            ws.connect(url).await;
            ws
        };
        self.state = WebsocketStage::Connecting(Box::pin(future));
    }
    pub async fn close_connection(&mut self) {
        match core::mem::replace(&mut self.state, WebsocketStage::Empty) {
            WebsocketStage::Idle(t) => {
                self.state = WebsocketStage::Idle(t);
            }
            WebsocketStage::Connecting(mut t) => {
                let mut res = t.as_mut().await;
                res.close().await;
                self.state = WebsocketStage::Idle(res);
            }
            WebsocketStage::Connected(mut t) => {
                t.close().await;
                self.state = WebsocketStage::Idle(t);
            }
            _ => {
                unreachable!();
            }
        }
    }
    pub async fn send_msg(&mut self, msg: String) {
        match &mut self.state {
            WebsocketStage::Connected(ws) => {
                ws.send(msg).await;
            }
            _ => {
                unreachable!();
            }
        }
    }
}

#[cfg(feature = "async")]
impl<W: WebsocketTransport> Future for WebsocketClient<W> {
    type Output = WebsocketResponse;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.state {
            WebsocketStage::Idle(_) => Poll::Ready(WebsocketResponse::Disconnected),
            WebsocketStage::Connecting(fut) => match fut.as_mut().poll(cx) {
                Poll::Ready(ws) => {
                    self.state = WebsocketStage::Connected(ws);
                    Poll::Ready(WebsocketResponse::Connected)
                }
                Poll::Pending => Poll::Pending,
            },
            WebsocketStage::Connected(ws) => match ws.poll_recv(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(t) => match t {
                    Some(t) => Poll::Ready(WebsocketResponse::WsMsg(t)),
                    None => Poll::Ready(WebsocketResponse::Disconnected),
                },
            },
            WebsocketStage::Empty => {
                unreachable!();
            }
        }
    }
}

#[cfg(feature = "tokio_ws")]
pub struct TokioWsClient {
    socket: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

#[cfg(feature = "tokio_ws")]
impl TokioWsClient {
    pub fn new() -> Self {
        Self { socket: None }
    }
}

#[cfg(feature = "tokio_ws")]
#[async_trait::async_trait]
impl WebsocketTransport for TokioWsClient {
    async fn connect(&mut self, url: String) {
        log::debug!("connecting to url: {}", url);
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