use ocpp_core::v16::types::{ChargePointErrorCode, ChargePointStatus, Measurand};

use crate::v16::interface::{MeterData, MeterDataType, Secc};

#[derive(Clone, PartialEq, Debug)]
pub enum SeccState {
    Plugged,
    Unplugged,
    Faulty,
}

#[derive(Debug)]
pub enum SeccActions {
    Secc(
        usize,
        SeccState,
        Option<ChargePointErrorCode>,
        Option<String>,
    ),
    IdTag(usize, String),
}

pub(crate) struct SeccService<S: Secc> {
    secc: S,
}

impl<S: Secc> SeccService<S> {
    pub fn new(secc: S) -> Self {
        Self { secc }
    }
    pub fn hard_reset(&self) {
        self.secc.hard_reset();
    }
    pub fn update_status(&self, connector_id: usize, status: ChargePointStatus) {
        self.secc.update_status(connector_id, status)
    }
    pub fn get_meter_value(&self, connector_id: usize, kind: &MeterDataType) -> Option<MeterData> {
        self.secc.get_meter_value(connector_id, kind)
    }
    pub fn get_start_stop_value(&self, connector_id: usize) -> u64 {
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
