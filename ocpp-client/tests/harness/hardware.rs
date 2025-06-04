use std::{future::Future, pin::Pin, task::{Context, Poll}, time::Duration};

use flume::{r#async::RecvFut, unbounded, Sender};
use rocpp_client::v16::{Hardware, HardwareEvent, MeterData, MeterDataType};
use rocpp_core::v16::types::ChargePointStatus;
use tokio_util::sync::CancellationToken;
use futures::FutureExt;

pub struct MockHardware {
    hard_reset_token: CancellationToken,
    ev_rx_fut: RecvFut<'static, HardwareEvent>,
    cancel_fut: Option<Pin<Box<dyn Future<Output = ()> + 'static>>>,
}

impl MockHardware {
    pub fn new(token: CancellationToken) -> (Self,  Sender<HardwareEvent>) {
        let (ev_tx, ev_rx) = unbounded();
        (Self {
            hard_reset_token: token,
            ev_rx_fut: ev_rx.into_recv_async(),
            cancel_fut: None
        }, ev_tx)
    }
}

impl Hardware for MockHardware {
    async fn get_boot_time(&self) -> u64 {
        uptime_lib::get().unwrap().as_micros() as u64
    }
    async fn hard_reset(&mut self) {
        let token = self.hard_reset_token.clone();
        tokio::task::spawn_local(async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            token.cancel();
        });
    }
    async fn update_status(&mut self, connector_id: usize, status: ChargePointStatus) {
        log::info!(
            "connector state for connector: {}, state: {:?}",
            connector_id,
            status
        );
    }
    async fn get_meter_value(&mut self, connector_id: usize, kind: &MeterDataType) -> Option<MeterData> {
        log::info!(
            "requested meter value for connector: {}, kind: {:?}",
            connector_id,
            kind
        );
        return Some(MeterData {
            value: String::from("10"),
            location: None,
            unit: None,
        });
    }
    fn poll_hardware_events(&mut self, cx: &mut Context<'_>) -> Poll<HardwareEvent> {
        match self.ev_rx_fut.poll_unpin(cx) {
            Poll::Ready(t) => Poll::Ready(t.unwrap()),
            Poll::Pending => Poll::Pending,
        }
    }
    fn poll_reset(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.cancel_fut.is_none() {
            let f = self.hard_reset_token.clone().cancelled_owned();
            self.cancel_fut = Some(Box::pin(f));
        }

        let fut = self.cancel_fut.as_mut().unwrap();
        match fut.poll_unpin(cx) {
            Poll::Ready(()) => {
                self.cancel_fut = None;
                Poll::Ready(())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
