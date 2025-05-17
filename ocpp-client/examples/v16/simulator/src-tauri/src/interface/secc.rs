use ocpp_client::v16::{MeterData, MeterDataType, Secc};
use ocpp_core::v16::types::{ChargePointStatus, Measurand};
use tokio_util::sync::CancellationToken;

use super::ui::UiClient;

pub struct SeccService {
    pub ui: UiClient,
    pub stop_token: CancellationToken,
}

impl SeccService {
    pub fn new(ui: UiClient, stop_token: CancellationToken) -> Self {
        Self { ui, stop_token }
    }
}

impl Secc for SeccService {
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
