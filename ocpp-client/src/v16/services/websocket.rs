use core::{
    future::Future,
    mem,
    pin::Pin,
    task::{Context, Poll},
};

use alloc::{boxed::Box, string::String};

use crate::v16::interface::WebsocketIo;

pub(crate) enum WebsocketStage<W: WebsocketIo> {
    Idle(W),
    Connecting(Pin<Box<dyn Future<Output = W> + Send>>),
    Connected(W),
    Empty,
}

#[derive(Debug)]
pub enum WebsocketResponse {
    Connected,
    Disconnected,
    WsMsg(String),
}

pub(crate) struct WebsocketService<W: WebsocketIo> {
    state: WebsocketStage<W>,
}

impl<W: WebsocketIo> WebsocketService<W> {
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

impl<W: WebsocketIo> Future for WebsocketService<W> {
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
