use alloc::{collections::btree_set::BTreeSet, string::String, vec::Vec};
use chrono::{DateTime, Utc};
use rocpp_core::v16::{
    messages::{
        meter_values::MeterValuesRequest, start_transaction::StartTransactionRequest,
        stop_transaction::StopTransactionRequest,
    },
    types::{MeterValue, Reason, SampledValue},
};
use serde::Serialize;

use crate::v16::{
    cp::ChargePoint,
    interfaces::{ChargePointInterface, SeccState},
};

use super::{call::CallAction, clock::Instant, connector::ConnectorState, firmware::FirmwareState};

#[derive(Clone)]
pub(crate) enum TransactionTime {
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
            None => Ok(TransactionTime::Unaligned(Instant::default())),
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct StartTransactionEvent {
    pub local_transaction_id: u32,
    pub connector_id: usize,
    pub id_tag: String,
    pub meter_start: u64,
    pub reservation_id: Option<i32>,
    pub timestamp: TransactionTime,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct MeterValueLocal {
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
pub(crate) struct MeterValuesEvent {
    pub connector_id: usize,
    pub local_transaction_id: Option<u32>,
    pub meter_value: Vec<MeterValueLocal>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct StopTransactionEvent {
    pub local_transaction_id: u32,
    pub id_tag: Option<String>,
    pub meter_stop: u64,
    pub timestamp: TransactionTime,
    pub reason: Option<Reason>,
    pub transaction_data: Option<Vec<MeterValueLocal>>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) enum TransactionEvent {
    Start(StartTransactionEvent),
    Meter(MeterValuesEvent),
    Stop(StopTransactionEvent),
}

#[derive(Clone)]
pub(crate) enum TransactionEventState {
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

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn pop_event(
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
                self.transaction_stop_meter_val_count
                    .remove(&local_transaction_id);
            }
        }
        self.interface
            .db_pop_transaction_event(
                self.transaction_tail,
                local_transaction_id,
                transaction_id,
                meter_tx,
            )
            .await;
        self.transaction_tail += 1;
        self.transaction_event_state = TransactionEventState::Idle;
        self.transaction_event_retries = 0;
        if self.transaction_tail != self.transaction_head {
            self.transacion_current_event = Some(
                self.interface
                    .db_get_transaction_event(self.transaction_tail)
                    .await,
            );
        } else {
            self.transacion_current_event = None;
        }
    }
    pub(crate) async fn process_transaction(&mut self) {
        loop {
            if self.call_permission() {
                if let (TransactionEventState::Idle, Some(tx)) = (
                    self.transaction_event_state.clone(),
                    self.transacion_current_event.clone(),
                ) {
                    match tx {
                        TransactionEvent::Start(t) => {
                            self.transaction_event_state =
                                TransactionEventState::WaitingForResponse;
                            self.transaction_event_retries += 1;
                            let req = self.get_start_transaction_request(t);
                            self.enqueue_call(CallAction::StartTransaction, req).await;
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
                                    self.enqueue_call(CallAction::MeterValues, req).await;
                                }
                                Err(_) => {
                                    //corresponsing transaction_id not found, droping
                                    assert!(self.transaction_event_retries == 0);
                                    self.pop_event(t.local_transaction_id, None, None).await;
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
                                self.enqueue_call(CallAction::StopTransaction, req).await;
                            } else {
                                //corresponsing transaction_id not found, droping
                                assert!(self.transaction_event_retries == 0);
                                let meter_tx = self
                                    .transaction_stop_meter_val_count
                                    .get(&t.local_transaction_id)
                                    .map(|f| *f)
                                    .unwrap_or(0);
                                self.pop_event(Some(t.local_transaction_id), None, Some(meter_tx))
                                    .await;
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
    pub(crate) async fn get_transaction_time(&self) -> TransactionTime {
        if let Some(date) = self.get_time().await {
            TransactionTime::Known(date)
        } else {
            TransactionTime::Unaligned(Instant::now(&self.interface).await)
        }
    }
    pub(crate) async fn add_transaction_event(&mut self, mut event: TransactionEvent) {
        match &mut event {
            TransactionEvent::Start(t) => {
                self.transaction_connector_map
                    .insert(t.local_transaction_id, t.connector_id);
            }
            TransactionEvent::Stop(t) => {
                if let Some(len) = self
                    .transaction_stop_meter_val_count
                    .get(&t.local_transaction_id)
                {
                    t.transaction_data = Some(
                        self.interface
                            .db_get_all_stop_meter_val(t.local_transaction_id, *len)
                            .await,
                    )
                }
            }
            _ => {}
        }
        self.interface
            .db_push_transaction_event(self.transaction_head, event)
            .await;

        self.transaction_head += 1;
        if self.transacion_current_event.is_none() {
            self.transacion_current_event = Some(
                self.interface
                    .db_get_transaction_event(self.transaction_tail)
                    .await,
            );
        }
        self.process_transaction().await;
    }
    pub(crate) async fn add_stop_transaction_meter_value(
        &mut self,
        local_transaction_id: u32,
        values: MeterValueLocal,
    ) {
        let index = if let Some(data) = self
            .transaction_stop_meter_val_count
            .get_mut(&local_transaction_id)
        {
            let len = *data;
            *data += 1;
            len
        } else {
            self.transaction_stop_meter_val_count
                .insert(local_transaction_id, 1);
            0
        };
        self.interface
            .db_add_stop_meter_val(local_transaction_id, index, values)
            .await;
    }
    pub(crate) async fn on_transaction_online(&mut self) {
        self.process_transaction().await;
    }
    pub(crate) async fn start_transaction(
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
        )
        .await;
        let meter_start = self
            .interface
            .get_start_stop_meter_value(connector_id)
            .await;
        let start_event = StartTransactionEvent {
            local_transaction_id,
            connector_id,
            id_tag,
            meter_start,
            reservation_id,
            timestamp: self.get_transaction_time().await,
        };
        self.start_meter_data(connector_id).await;
        self.add_transaction_event(TransactionEvent::Start(start_event))
            .await;
    }

    pub(crate) async fn stop_transaction(
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
                let meter_stop = self
                    .interface
                    .get_start_stop_meter_value(connector_id)
                    .await;
                let stop_event = StopTransactionEvent {
                    local_transaction_id: *local_transaction_id,
                    id_tag,
                    meter_stop,
                    timestamp: self.get_transaction_time().await,
                    reason,
                    transaction_data: None,
                };
                (new_state, TransactionEvent::Stop(stop_event))
            }
            _ => {
                unreachable!();
            }
        };
        self.stop_meter_data(connector_id).await;
        self.change_connector_state(connector_id, new_state).await;
        self.add_transaction_event(stop_event).await;
        if self.active_local_transactions.iter().all(|f| f.is_none()) {
            if let FirmwareState::WaitingForTransactionToFinish = &self.firmware_state {
                self.try_firmware_install().await;
            }
        }
    }

    pub(crate) async fn deauthorize_transaction(&mut self, local_transaction_id: u32) {
        if let Some(connector_id) = self.transaction_connector_map.get(&local_transaction_id) {
            if let ConnectorState::Transaction {
                local_transaction_id: local_transaction_id_tx,
                is_evse_suspended,
                ..
            } = &mut self.connector_state[*connector_id]
            {
                if *local_transaction_id_tx == local_transaction_id {
                    if self.configs.stop_transaction_on_invalid_id.value {
                        self.stop_transaction(*connector_id, None, Some(Reason::DeAuthorized))
                            .await;
                    } else {
                        *is_evse_suspended = true;
                        self.sync_connector_states(*connector_id, None, None).await;
                    }
                }
            }
        }
    }
    pub(crate) async fn handle_unfinished_transactions(
        &mut self,
        unfinished_transactions: BTreeSet<u32>,
    ) {
        for local_transaction_id in unfinished_transactions {
            if let Some(connector_id) = self.transaction_connector_map.get(&local_transaction_id) {
                let meter_stop = self
                    .interface
                    .get_start_stop_meter_value(*connector_id)
                    .await;
                let stop_event = TransactionEvent::Stop(StopTransactionEvent {
                    local_transaction_id,
                    id_tag: None,
                    meter_stop,
                    timestamp: self.get_transaction_time().await,
                    reason: Some(Reason::PowerLoss),
                    transaction_data: None,
                });
                self.add_transaction_event(stop_event).await;
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
