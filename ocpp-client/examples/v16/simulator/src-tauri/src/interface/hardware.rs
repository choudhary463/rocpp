use std::{future::Future, pin::Pin, task::{Context, Poll}, time::Duration};

use flume::{r#async::RecvFut, Receiver};
use futures_util::FutureExt;
use rocpp_client::v16::{Hardware, HardwareEvent, MeterData, MeterDataType};
use rocpp_core::v16::types::{ChargePointStatus, Measurand};
use tokio_util::sync::CancellationToken;

use super::ui::UiClient;

pub struct HardwareService {
    pub ui: UiClient,
    pub stop_token: CancellationToken,
    ev_rx_fut: RecvFut<'static, HardwareEvent>,
    cancel_fut: Option<Pin<Box<dyn Future<Output = ()> + 'static>>>,
}

impl HardwareService {
    pub fn new(ui: UiClient, stop_token: CancellationToken, ev_rx: Receiver<HardwareEvent>) -> Self {
        Self { ui, stop_token, ev_rx_fut: ev_rx.into_recv_async(), cancel_fut: None }
    }
}

impl Hardware for HardwareService {
    async fn get_boot_time(&self) -> u64 {
        uptime_lib::get().unwrap().as_micros() as u64
    }
    async fn hard_reset(&mut self) {
        let token = self.stop_token.clone();
        tokio::task::spawn_local(async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            token.cancel();
        });
    }
    async fn update_status(&mut self, connector_id: usize, status: ChargePointStatus) {
        self.ui.update_connector_state(connector_id + 1, status);
    }
    async fn get_meter_value(&mut self, _connector_id: usize, kind: &MeterDataType) -> Option<MeterData> {
        match &kind.measurand {
            Measurand::EnergyActiveImportRegister => Some(MeterData {
                value: format!("10"),
                location: None,
                unit: None,
            }),
            _ => None,
        }
    }
    fn poll_hardware_events(&mut self, cx: &mut Context<'_>) -> Poll<HardwareEvent> {
        match self.ev_rx_fut.poll_unpin(cx) {
            Poll::Ready(t) => Poll::Ready(t.unwrap()),
            Poll::Pending => Poll::Pending,
        }
    }
    fn poll_reset(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.cancel_fut.is_none() {
            let f = self.stop_token.clone().cancelled_owned();
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
