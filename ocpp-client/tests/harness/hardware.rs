use ocpp_client::v16::{MeterData, MeterDataType, HardwareInterface};
use ocpp_core::v16::types::ChargePointStatus;
use tokio_util::sync::CancellationToken;

pub struct MockHardware {
    hard_reset_toekn: CancellationToken,
}

impl MockHardware {
    pub fn new(token: CancellationToken) -> Self {
        Self {
            hard_reset_toekn: token,
        }
    }
}

impl HardwareInterface for MockHardware {
    fn get_boot_time(&self) -> u128 {
        uptime_lib::get().unwrap().as_micros()
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
