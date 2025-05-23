use std::time::Instant;

use ocpp_client::v16::{MeterData, MeterDataType, Secc};
use ocpp_core::v16::types::ChargePointStatus;
use tokio_util::sync::CancellationToken;

pub struct MockSecc {
    hard_reset_toekn: CancellationToken,
}

impl MockSecc {
    pub fn new(token: CancellationToken) -> Self {
        Self {
            hard_reset_toekn: token,
        }
    }
}

impl Secc for MockSecc {
    fn get_boot_time(&self) -> u128 {
        Instant::now().elapsed().as_micros()
    }
    fn hard_reset(&self) {
        self.hard_reset_toekn.cancel();
    }
    fn update_status(&self, connector_id: usize, status: ChargePointStatus) {
        log::info!(
            "connector state for connector: {}, state: {:?}",
            connector_id,
            status
        );
    }
    fn get_meter_value(&self, connector_id: usize, kind: &MeterDataType) -> Option<MeterData> {
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
}
