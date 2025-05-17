use chrono::Timelike;
use ocpp_core::v16::types::{ReadingContext, SampledValue};

use crate::v16::{
    interface::{Database, MeterDataType, Secc},
    services::timeout::TimerId,
};

use super::{
    core::ChargePointCore,
    transaction::{MeterValueLocal, MeterValuesEvent, TransactionEvent},
};

#[derive(Clone)]
pub(crate) enum MeterState {
    Idle,
    Sleep,
}

pub(crate) enum MeterDataKind {
    MeterValuesSampled,
    StopTxnSampled,
    MeterValuesAligned,
    StopTxnAligned,
}

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn set_sampled_meter_sleep_state(&mut self, connector_id: usize) {
        self.add_timeout(
            TimerId::MeterSampled(connector_id),
            self.configs.meter_value_sample_interval.value,
        );
        self.sampled_meter_state[connector_id] = MeterState::Sleep;
    }
    pub fn set_aligned_meter_sleep_state(&mut self) {
        if self.configs.clock_aligned_data_interval.value > 0 {
            if let Some(time) = self.get_time() {
                let seconds = time.num_seconds_from_midnight() as u64;
                let rem = self.configs.clock_aligned_data_interval.value
                    - seconds % self.configs.clock_aligned_data_interval.value;
                self.aligned_meter_state = MeterState::Sleep;
                self.add_timeout(TimerId::MeterAligned, rem);
            } else {
                unreachable!();
            }
        }
    }
    pub fn start_meter_data(&mut self, connector_id: usize) {
        if self.configs.meter_value_sample_interval.value > 0 {
            match &self.sampled_meter_state[connector_id] {
                MeterState::Idle => {
                    self.set_sampled_meter_sleep_state(connector_id);
                }
                _ => {
                    unreachable!();
                }
            }
        }
    }
    pub fn stop_meter_data(&mut self, connector_id: usize) {
        if let MeterState::Sleep = &self.sampled_meter_state[connector_id] {
            self.remove_timeout(TimerId::MeterSampled(connector_id));
        }
    }
    pub fn add_meter_event(
        &mut self,
        connector_id: usize,
        local_transaction_id: Option<u32>,
        kind: MeterDataKind,
        context: ReadingContext,
    ) {
        let sampled_value = self.get_sampled_data(connector_id, kind, context);
        if !sampled_value.is_empty() {
            let meter_value_local = MeterValueLocal {
                timestamp: self.get_transaction_time(),
                sampled_value,
            };
            let meter_event = MeterValuesEvent {
                connector_id,
                local_transaction_id,
                meter_value: vec![meter_value_local],
            };
            self.add_transaction_event(TransactionEvent::Meter(meter_event));
        }
    }
    pub fn add_stop_transaction_sampled_data(
        &mut self,
        connector_id: usize,
        local_transaction_id: u32,
        kind: MeterDataKind,
        context: ReadingContext,
    ) {
        let sampled_value = self.get_sampled_data(connector_id, kind, context);
        if !sampled_value.is_empty() {
            let values = MeterValueLocal {
                timestamp: self.get_transaction_time(),
                sampled_value,
            };
            self.add_stop_transaction_meter_value(local_transaction_id, values);
        }
    }
    pub fn trigger_meter_values(&mut self, connector_id: usize) {
        for connector_id in if connector_id == 0 {
            0..self.configs.number_of_connectors.value
        } else {
            (connector_id - 1)..(connector_id)
        } {
            self.add_meter_event(
                connector_id,
                None,
                MeterDataKind::MeterValuesSampled,
                ReadingContext::Trigger,
            );
        }
    }
    fn get_measurands(&self, kind: MeterDataKind) -> &Vec<MeterDataType> {
        match kind {
            MeterDataKind::MeterValuesSampled => &self.configs.meter_values_sampled_data.value,
            MeterDataKind::StopTxnSampled => &self.configs.stop_transaction_sampled_data.value,
            MeterDataKind::MeterValuesAligned => &self.configs.meter_values_aligned_data.value,
            MeterDataKind::StopTxnAligned => &self.configs.stop_transaction_aligned_data.value,
        }
    }
    fn get_sampled_data(
        &mut self,
        connector_id: usize,
        kind: MeterDataKind,
        context: ReadingContext,
    ) -> Vec<SampledValue> {
        let mut sampled_value = Vec::new();
        for measurand in self.get_measurands(kind) {
            if let Some(res) = self.secc.get_meter_value(connector_id, measurand) {
                sampled_value.push(SampledValue {
                    value: res.value,
                    context: Some(context.clone()),
                    format: None,
                    measurand: Some(measurand.measurand.clone()),
                    phase: measurand.phase.clone(),
                    location: res.location,
                    unit: res.unit,
                });
            }
        }
        sampled_value
    }
}
