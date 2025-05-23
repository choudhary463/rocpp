use ocpp_core::v16::types::{ChargePointStatus, Measurand};

use crate::v16::interface::{MeterData, MeterDataType, Secc};

pub struct SeccService<S: Secc> {
    secc: S,
}

impl<S: Secc> SeccService<S> {
    pub fn new(secc: S) -> Self {
        Self { secc }
    }
    pub(crate) fn get_boot_time(&self) -> u128 {
        self.secc.get_boot_time()
    }
    pub(crate) fn hard_reset(&self) {
        self.secc.hard_reset();
    }
    pub(crate) fn update_status(&self, connector_id: usize, status: ChargePointStatus) {
        self.secc.update_status(connector_id, status)
    }
    pub(crate) fn get_meter_value(&self, connector_id: usize, kind: &MeterDataType) -> Option<MeterData> {
        self.secc.get_meter_value(connector_id, kind)
    }
    pub(crate) fn get_start_stop_value(&self, connector_id: usize) -> u64 {
        self.get_meter_value(
            connector_id,
            &MeterDataType {
                measurand: Measurand::EnergyActiveImportRegister,
                phase: None,
            },
        )
        .unwrap()
        .value
        .parse()
        .unwrap()
    }
}
