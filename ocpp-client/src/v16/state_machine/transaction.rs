use std::{collections::HashSet, time::Instant};

use chrono::{DateTime, Utc};
use ocpp_core::v16::{
    messages::{
        meter_values::MeterValuesRequest, start_transaction::StartTransactionRequest,
        stop_transaction::StopTransactionRequest,
    },
    types::{MeterValue, Reason, SampledValue},
};
use serde::Serialize;

use crate::v16::{
    interface::{Database, Secc},
    services::secc::SeccState,
};

use super::{
    call::CallAction, connector::ConnectorState, core::ChargePointCore, firmware::FirmwareState,
};

#[derive(Clone)]
pub enum TransactionTime {
    Unaligned(Instant),
    Known(DateTime<Utc>),
}

impl Serialize for TransactionTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            TransactionTime::Known(dt) => dt.serialize(serializer),
            TransactionTime::Unaligned(_) => serializer.serialize_none(),
        }
    }
}

impl<'de> serde::de::Deserialize<'de> for TransactionTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let opt = Option::<DateTime<Utc>>::deserialize(deserializer)?;
        match opt {
            Some(dt) => Ok(TransactionTime::Known(dt)),
            None => Ok(TransactionTime::Unaligned(Instant::now())),
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct StartTransactionEvent {
    pub local_transaction_id: u32,
    pub connector_id: usize,
    pub id_tag: String,
    pub meter_start: u64,
    pub reservation_id: Option<i32>,
    pub timestamp: TransactionTime,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct MeterValueLocal {
    pub timestamp: TransactionTime,
    pub sampled_value: Vec<SampledValue>,
}

impl MeterValueLocal {
    pub fn into_meter_value(self, timestamp: DateTime<Utc>) -> MeterValue {
        MeterValue {
            timestamp,
            sampled_value: self.sampled_value,
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct MeterValuesEvent {
    pub connector_id: usize,
    pub local_transaction_id: Option<u32>,
    pub meter_value: Vec<MeterValueLocal>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct StopTransactionEvent {
    pub local_transaction_id: u32,
    pub id_tag: Option<String>,
    pub meter_stop: u64,
    pub timestamp: TransactionTime,
    pub reason: Option<Reason>,
    pub transaction_data: Option<Vec<MeterValueLocal>>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum TransactionEvent {
    Start(StartTransactionEvent),
    Meter(MeterValuesEvent),
    Stop(StopTransactionEvent),
}

#[derive(Clone)]
pub enum TransactionEventState {
    Idle,
    Sleeping,
    WaitingForResponse,
}

impl TransactionEvent {
    pub fn is_stop(&self) -> bool {
        matches!(self, TransactionEvent::Stop(_))
    }
    pub fn get_local_transaction_id(&self) -> Option<u32> {
        match self {
            TransactionEvent::Start(t) => Some(t.local_transaction_id),
            TransactionEvent::Meter(t) => t.local_transaction_id,
            TransactionEvent::Stop(t) => Some(t.local_transaction_id),
        }
    }
}

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn pop_event(
        &mut self,
        local_transaction_id: Option<u32>,
        transaction_id: Option<i32>,
        meter_tx: Option<usize>,
    ) {
        if let Some(local_transaction_id) = local_transaction_id {
            if let Some(transaction_id) = transaction_id {
                self.transaction_map
                    .insert(local_transaction_id, transaction_id);
                if let Some(connector_id) = self
                    .active_local_transactions
                    .iter()
                    .position(|f| f.map(|t| t.0 == local_transaction_id).unwrap_or(false))
                {
                    self.active_local_transactions[connector_id] =
                        Some((local_transaction_id, Some(transaction_id)));
                }
            }
            if meter_tx.is_some() {
                self.transaction_map.remove(&local_transaction_id);
                self.transaction_connector_map.remove(&local_transaction_id);
                self.transaction_stop_meter_map
                    .remove(&local_transaction_id);
            }
        }
        let _ = self.transaction_queue.pop_front();
        self.db.db_pop_transaction_event(
            self.transaction_tail,
            local_transaction_id,
            transaction_id,
            meter_tx,
        );
        self.transaction_tail += 1;
        self.transaction_event_state = TransactionEventState::Idle;
        self.transaction_event_retries = 0;
        self.process_transaction();
    }
    pub fn process_transaction(&mut self) {
        loop {
            if self.call_permission() {
                if let (TransactionEventState::Idle, Some(tx)) = (
                    self.transaction_event_state.clone(),
                    self.transaction_queue.front().cloned(),
                ) {
                    match tx {
                        TransactionEvent::Start(t) => {
                            self.transaction_event_state =
                                TransactionEventState::WaitingForResponse;
                            self.transaction_event_retries += 1;
                            let req = self.get_start_transaction_request(t);
                            self.enqueue_call(CallAction::StartTransaction, req);
                        }
                        TransactionEvent::Meter(t) => {
                            match t.local_transaction_id.map_or(Ok(None), |key| {
                                self.transaction_map
                                    .get(&key)
                                    .map(|v| Ok(Some(*v)))
                                    .unwrap_or_else(|| Err(()))
                            }) {
                                Ok(transaction_id) => {
                                    self.transaction_event_state =
                                        TransactionEventState::WaitingForResponse;
                                    self.transaction_event_retries += 1;
                                    let req = self.get_meter_values_request(t, transaction_id);
                                    self.enqueue_call(CallAction::MeterValues, req);
                                }
                                Err(_) => {
                                    //corresponsing transaction_id not found, droping
                                    assert!(self.transaction_event_retries == 0);
                                    self.pop_event(t.local_transaction_id, None, None);
                                }
                            }
                        }
                        TransactionEvent::Stop(t) => {
                            if let Some(transaction_id) =
                                self.transaction_map.get(&t.local_transaction_id)
                            {
                                self.transaction_event_state =
                                    TransactionEventState::WaitingForResponse;
                                self.transaction_event_retries += 1;
                                let req = self.get_stop_transaction_request(t, *transaction_id);
                                self.enqueue_call(CallAction::StopTransaction, req);
                            } else {
                                //corresponsing transaction_id not found, droping
                                assert!(self.transaction_event_retries == 0);
                                let meter_tx = self
                                    .transaction_stop_meter_map
                                    .get(&t.local_transaction_id)
                                    .map(|f| f.len())
                                    .unwrap_or(0);
                                self.pop_event(Some(t.local_transaction_id), None, Some(meter_tx));
                            }
                        }
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
    pub fn get_transaction_time(&self) -> TransactionTime {
        if let Some(date) = self.get_time() {
            TransactionTime::Known(date)
        } else {
            TransactionTime::Unaligned(Instant::now())
        }
    }
    pub fn add_transaction_event(&mut self, mut event: TransactionEvent) {
        match &mut event {
            TransactionEvent::Start(t) => {
                self.transaction_connector_map
                    .insert(t.local_transaction_id, t.connector_id);
            }
            TransactionEvent::Stop(t) => {
                t.transaction_data = self
                    .transaction_stop_meter_map
                    .get(&t.local_transaction_id)
                    .cloned()
            }
            _ => {}
        }
        self.transaction_queue.push_back(event.clone());
        self.db
            .db_push_transaction_event(self.transaction_head, event);
        self.transaction_head += 1;
        self.process_transaction();
    }
    pub fn add_stop_transaction_meter_value(
        &mut self,
        local_transaction_id: u32,
        values: MeterValueLocal,
    ) {
        let index = if let Some(data) = self
            .transaction_stop_meter_map
            .get_mut(&local_transaction_id)
        {
            let len = data.len();
            data.push(values.clone());
            len
        } else {
            self.transaction_stop_meter_map
                .insert(local_transaction_id, vec![values.clone()]);
            0
        };
        self.db.db_add_meter_tx(local_transaction_id, index, values);
    }
    pub fn on_transaction_online(&mut self) {
        self.process_transaction();
    }
    pub fn start_transaction(
        &mut self,
        connector_id: usize,
        id_tag: String,
        parent_id_tag: Option<String>,
        reservation_id: Option<i32>,
    ) {
        let local_transaction_id = self.local_transaction_id + 1;
        self.local_transaction_id += 1;
        self.active_local_transactions[connector_id] = Some((local_transaction_id, None));
        self.change_connector_state(
            connector_id,
            ConnectorState::transaction(
                local_transaction_id,
                id_tag.clone(),
                parent_id_tag.clone(),
                false,
                SeccState::Plugged,
            ),
        );
        let meter_start = self.secc.get_start_stop_value(connector_id);
        let start_event = StartTransactionEvent {
            local_transaction_id,
            connector_id,
            id_tag,
            meter_start,
            reservation_id,
            timestamp: self.get_transaction_time(),
        };
        self.start_meter_data(connector_id);
        self.add_transaction_event(TransactionEvent::Start(start_event));
    }

    pub fn stop_transaction(
        &mut self,
        connector_id: usize,
        id_tag: Option<String>,
        reason: Option<Reason>,
    ) {
        let (new_state, stop_event) = match &self.connector_state[connector_id] {
            ConnectorState::Transaction {
                secc_state,
                local_transaction_id,
                ..
            } => {
                let is_unavailable = self.pending_inoperative_changes[connector_id];
                let new_state = if is_unavailable {
                    self.pending_inoperative_changes[connector_id] = false;
                    ConnectorState::Unavailable(secc_state.clone())
                } else {
                    match &secc_state {
                        SeccState::Faulty => ConnectorState::faulty(),
                        SeccState::Plugged => ConnectorState::finishing(),
                        SeccState::Unplugged => ConnectorState::idle(),
                    }
                };
                self.active_local_transactions[connector_id] = None;
                let meter_stop = self.secc.get_start_stop_value(connector_id);
                let stop_event = StopTransactionEvent {
                    local_transaction_id: *local_transaction_id,
                    id_tag,
                    meter_stop,
                    timestamp: self.get_transaction_time(),
                    reason,
                    transaction_data: None,
                };
                (new_state, TransactionEvent::Stop(stop_event))
            }
            _ => {
                unreachable!();
            }
        };
        self.stop_meter_data(connector_id);
        self.change_connector_state(connector_id, new_state);
        self.add_transaction_event(stop_event);
        if self.active_local_transactions.iter().all(|f| f.is_none()) {
            if let FirmwareState::WaitingForTransactionToFinish(firmware_image) = &self.firmware_state {
                self.try_firmware_install(firmware_image.clone());
            }
        }
    }

    pub fn deauthorize_transaction(&mut self, local_transaction_id: u32) {
        if let Some(connector_id) = self.transaction_connector_map.get(&local_transaction_id) {
            if let ConnectorState::Transaction {
                local_transaction_id: local_transaction_id_tx,
                is_evse_suspended,
                ..
                } = &mut self.connector_state[*connector_id] {
                if *local_transaction_id_tx == local_transaction_id {
                    if self.configs.stop_transaction_on_invalid_id.value {
                        self.stop_transaction(*connector_id, None, Some(Reason::DeAuthorized));
                    } else {
                        *is_evse_suspended = true;
                        self.sync_connector_states(*connector_id, None, None);
                    }
                }
            }
        }
    }
    pub fn handle_unfinished_transactions(&mut self) {
        let mut unfinished_txn = HashSet::new();
        for local_transaction_id in self.transaction_connector_map.keys() {
            unfinished_txn.insert(*local_transaction_id);
        }
        for event in self.transaction_queue.iter() {
            if let Some(id) = event.get_local_transaction_id() {
                if !event.is_stop() {
                    unfinished_txn.insert(id);
                } else {
                    unfinished_txn.remove(&id);
                }
            }
        }
        for local_transaction_id in unfinished_txn {
            if let Some(connector_id) = self.transaction_connector_map.get(&local_transaction_id) {
                let meter_stop = self.secc.get_start_stop_value(*connector_id);
                let stop_event = TransactionEvent::Stop(StopTransactionEvent {
                    local_transaction_id,
                    id_tag: None,
                    meter_stop,
                    timestamp: self.get_transaction_time(),
                    reason: Some(Reason::PowerLoss),
                    transaction_data: None,
                });
                self.add_transaction_event(stop_event);
            }
        }
    }
    fn get_start_transaction_request(
        &self,
        event: StartTransactionEvent,
    ) -> StartTransactionRequest {
        StartTransactionRequest {
            connector_id: event.connector_id + 1,
            id_tag: event.id_tag,
            meter_start: event.meter_start,
            reservation_id: event.reservation_id,
            timestamp: self.parse_transaction_time(event.timestamp),
        }
    }
    fn get_meter_values_request(
        &self,
        event: MeterValuesEvent,
        transaction_id: Option<i32>,
    ) -> MeterValuesRequest {
        MeterValuesRequest {
            connector_id: event.connector_id + 1,
            transaction_id,
            meter_value: event
                .meter_value
                .into_iter()
                .map(|f| {
                    let timestamp = self.parse_transaction_time(f.timestamp.clone());
                    f.into_meter_value(timestamp)
                })
                .collect(),
        }
    }
    fn get_stop_transaction_request(
        &self,
        event: StopTransactionEvent,
        transaction_id: i32,
    ) -> StopTransactionRequest {
        StopTransactionRequest {
            id_tag: event.id_tag,
            meter_stop: event.meter_stop,
            timestamp: self.parse_transaction_time(event.timestamp),
            transaction_id,
            reason: event.reason,
            transaction_data: event.transaction_data.map(|t| {
                t.into_iter()
                    .map(|f| {
                        let timestamp = self.parse_transaction_time(f.timestamp.clone());
                        f.into_meter_value(timestamp)
                    })
                    .collect()
            }),
        }
    }
    fn parse_transaction_time(&self, time: TransactionTime) -> DateTime<Utc> {
        match time {
            TransactionTime::Known(t) => t,
            TransactionTime::Unaligned(t) => self.get_time_since(t).unwrap_or(self.default_time()),
        }
    }
}
