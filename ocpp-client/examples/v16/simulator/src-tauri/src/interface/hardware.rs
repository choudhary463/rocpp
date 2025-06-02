use ocpp_client::v16::{MeterData, MeterDataType, HardwareInterface};
use ocpp_core::v16::types::{ChargePointStatus, Measurand};
use tokio_util::sync::CancellationToken;

use super::ui::UiClient;

pub struct HardwareService {
    pub ui: UiClient,
    pub stop_token: CancellationToken,
}

impl HardwareService {
    pub fn new(ui: UiClient, stop_token: CancellationToken) -> Self {
        Self { ui, stop_token }
    }
}

impl HardwareInterface for HardwareService {
    fn get_boot_time(&self) -> u128 {
        uptime_lib::get().unwrap().as_micros()
    }
    fn hard_reset(&self) {
        self.stop_token.cancel();
    }
    fn update_status(&self, connector_id: usize, status: ChargePointStatus) {
        self.ui.update_connector_state(connector_id + 1, status);
    }
    fn get_meter_value(&self, _connector_id: usize, kind: &MeterDataType) -> Option<MeterData> {
        match &kind.measurand {
            Measurand::EnergyActiveImportRegister => Some(MeterData {
                value: format!("10"),
                location: None,
                unit: None,
            }),
            _ => None,
        }
    }
}
