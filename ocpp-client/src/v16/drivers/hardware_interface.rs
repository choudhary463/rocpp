use alloc::string::String;
use ocpp_core::v16::types::{ChargePointStatus, Location, Measurand, Phase, UnitOfMeasure};

#[derive(Debug, Clone, PartialEq)]
pub struct MeterDataType {
    pub measurand: Measurand,
    pub phase: Option<Phase>,
}

pub struct MeterData {
    pub value: String,
    pub location: Option<Location>,
    pub unit: Option<UnitOfMeasure>,
}

pub trait HardwareInterface: Send + 'static {
    fn get_boot_time(&self) -> u128;
    fn hard_reset(&self);
    fn update_status(&self, connector_id: usize, status: ChargePointStatus);
    fn get_meter_value(&self, connector_id: usize, kind: &MeterDataType) -> Option<MeterData>;
}

pub struct HardwareBridge<HW: HardwareInterface> {
    hw: HW,
}

impl<HW: HardwareInterface> HardwareBridge<HW> {
    pub fn new(hw: HW) -> Self {
        Self { hw }
    }
    pub(crate) fn get_boot_time(&self) -> u128 {
        self.hw.get_boot_time()
    }
    pub(crate) fn hard_reset(&self) {
        self.hw.hard_reset();
    }
    pub(crate) fn update_status(&self, connector_id: usize, status: ChargePointStatus) {
        self.hw.update_status(connector_id, status)
    }
    pub(crate) fn get_meter_value(&self, connector_id: usize, kind: &MeterDataType) -> Option<MeterData> {
        self.hw.get_meter_value(connector_id, kind)
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
