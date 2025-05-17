use std::{
    future::Future,
    mem,
    pin::Pin,
    task::{Context, Poll},
};

use crate::v16::interface::WebsocketIo;

pub(crate) enum WebsocketState<W: WebsocketIo> {
    Idle(W),
    Connecting(Pin<Box<dyn Future<Output = W> + Send>>),
    Connected(W),
    Empty,
}

#[derive(Debug)]
pub(crate) enum WebsocketResponse {
    Connected,
    Disconnected,
    WsMsg(String),
}

pub(crate) struct WebsocketService<W: WebsocketIo> {
    state: WebsocketState<W>,
}

impl<W: WebsocketIo> WebsocketService<W> {
    pub fn new(ws: W) -> Self {
        Self {
            state: WebsocketState::Idle(ws),
        }
    }

    pub fn connect(&mut self, url: String) {
        let old_state = mem::replace(&mut self.state, WebsocketState::Empty);

        let ws = match old_state {
            WebsocketState::Idle(ws) => ws,
            WebsocketState::Connected(ws) => ws,
            _ => {
                unreachable!();
            }
        };

        let future = async move {
            let mut ws = ws;
            ws.connect(url).await;
            ws
        };
        self.state = WebsocketState::Connecting(Box::pin(future));
    }
    pub async fn close_connection(&mut self) {
        match std::mem::replace(&mut self.state, WebsocketState::Empty) {
            WebsocketState::Idle(t) => {
                self.state = WebsocketState::Idle(t);
            }
            WebsocketState::Connecting(mut t) => {
                let mut res = t.as_mut().await;
                res.close().await;
                self.state = WebsocketState::Idle(res);
            }
            WebsocketState::Connected(mut t) => {
                t.close().await;
                self.state = WebsocketState::Idle(t);
            }
            _ => {
                unreachable!();
            }
        }
    }
    pub async fn send_msg(&mut self, msg: String) {
        match &mut self.state {
            WebsocketState::Connected(ws) => {
                ws.send(msg).await;
            }
            _ => {
                unreachable!();
            }
        }
    }
}

impl<W: WebsocketIo> Future for WebsocketService<W> {
    type Output = WebsocketResponse;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.state {
            WebsocketState::Idle(_) => Poll::Ready(WebsocketResponse::Disconnected),
            WebsocketState::Connecting(fut) => match fut.as_mut().poll(cx) {
                Poll::Ready(ws) => {
                    self.state = WebsocketState::Connected(ws);
                    Poll::Ready(WebsocketResponse::Connected)
                }
                Poll::Pending => Poll::Pending,
            },
            WebsocketState::Connected(ws) => match ws.poll_recv(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(t) => match t {
                    Some(t) => Poll::Ready(WebsocketResponse::WsMsg(t)),
                    None => Poll::Ready(WebsocketResponse::Disconnected),
                },
            },
            WebsocketState::Empty => {
                unreachable!();
            }
        }
    }
}
