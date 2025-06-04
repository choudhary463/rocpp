use alloc::{
    collections::{BTreeMap, BTreeSet},
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use rocpp_core::v16::{
    messages::{reserve_now::ReserveNowRequest, status_notification::StatusNotificationRequest},
    types::{AvailabilityType, ChargePointErrorCode, IdTagInfo, Measurand},
};

use crate::v16::state_machine::{
    auth::LocalListChange,
    connector::ConnectorState,
    firmware::FirmwareInstallStatus,
    transaction::{MeterValueLocal, TransactionEvent},
};

use super::{ChargePointInterface, MeterDataType, SeccState};

pub(crate) struct ChargePointBackend<I: ChargePointInterface> {
    pub interface: I,
}

impl<I: ChargePointInterface> ChargePointBackend<I> {
    pub fn new(interface: I) -> Self {
        Self { interface }
    }
    pub async fn get_start_stop_meter_value(&mut self, connector_id: usize) -> u64 {
        self.interface
            .get_meter_value(
                connector_id,
                &MeterDataType {
                    measurand: Measurand::EnergyActiveImportRegister,
                    phase: None,
                },
            )
            .await
            .unwrap()
            .value
            .parse()
            .unwrap()
    }
    pub async fn init(&mut self, default_configs: Vec<(String, String)>, clear_db: bool) {
        self.interface.db_init().await;
        let mut previous_configs = self.interface.db_get_all("previous_configs").await;
        let mut default_configs: Vec<_> = default_configs
            .iter()
            .map(|t| (t.0.as_str(), t.1.as_str()))
            .collect();
        previous_configs.sort();
        default_configs.sort();
        if clear_db || default_configs != previous_configs {
            self.interface.db_delete_all().await;
            self.interface
                .db_transaction(
                    "previous_configs",
                    default_configs
                        .clone()
                        .into_iter()
                        .map(|(key, value)| (key, Some(value)))
                        .collect(),
                )
                .await;
            self.interface
                .db_transaction(
                    "config",
                    default_configs
                        .into_iter()
                        .map(|(key, value)| (key, Some(value)))
                        .collect(),
                )
                .await;
        }
    }

    pub async fn db_get_all_configs(&mut self) -> Vec<(&str, &str)> {
        self.interface.db_get_all("config").await
    }
    async fn db_get_reservations(&mut self) -> Vec<ReserveNowRequest> {
        self.interface
            .db_get_all("reservation")
            .await
            .into_iter()
            .map(|f| serde_json::from_str::<ReserveNowRequest>(&f.1).unwrap())
            .collect()
    }
    async fn db_get_operative_state(&mut self, num_connectors: usize) -> Vec<AvailabilityType> {
        let mut availability = vec![AvailabilityType::Operative; num_connectors];
        self.interface
            .db_get_all("availabilitytype")
            .await
            .iter()
            .for_each(|(key, value)| {
                let connector_id: usize = key.parse().unwrap();
                let kind = serde_json::from_str::<AvailabilityType>(&value).unwrap();
                availability[connector_id] = kind;
            });
        availability
    }
    pub async fn db_change_operative_state(
        &mut self,
        connector_id: usize,
        state: AvailabilityType,
    ) {
        let key = connector_id.to_string();
        let value = serde_json::to_string(&state).unwrap();
        self.interface
            .db_transaction(
                "availabilitytype",
                vec![(key.as_str(), Some(value.as_str()))],
            )
            .await;
    }
    pub async fn db_get_connector_state(
        &mut self,
        num_connectors: usize,
    ) -> (Vec<ConnectorState>, Vec<StatusNotificationRequest>) {
        let mut connector_state: Vec<_> = self
            .db_get_operative_state(num_connectors)
            .await
            .into_iter()
            .map(|f| match f {
                AvailabilityType::Operative => ConnectorState::Idle,
                AvailabilityType::Inoperative => ConnectorState::Unavailable(SeccState::Unplugged),
            })
            .collect();
        let reservations = self.db_get_reservations().await;
        for reservation in reservations {
            let connector_id = reservation.connector_id;
            connector_state[connector_id] = ConnectorState::reserved(
                reservation.reservation_id,
                reservation.id_tag,
                reservation.parent_id_tag,
                false,
            );
        }
        let mut status = Vec::new();
        for connector_id in 0..num_connectors {
            status.push(StatusNotificationRequest {
                connector_id: (connector_id + 1),
                error_code: ChargePointErrorCode::NoError,
                info: None,
                status: connector_state[connector_id].get_connector_state(false),
                timestamp: None,
                vendor_id: None,
                vendor_error_code: None,
            });
        }
        (connector_state, status)
    }

    pub async fn db_get_transaction_data(
        &mut self,
    ) -> (
        u32,
        u64,
        u64,
        BTreeMap<u32, i32>,
        BTreeMap<u32, usize>,
        BTreeMap<u32, usize>,
        BTreeSet<u32>,
    ) {
        let mut transaction_map: BTreeMap<u32, i32> = BTreeMap::new();
        let mut transaction_connector_map: BTreeMap<u32, usize> = BTreeMap::new();
        let mut transaction_stop_meter_val_count: BTreeMap<u32, usize> = BTreeMap::new();
        let mut unfinished_transactions: BTreeSet<u32> = BTreeSet::new();
        let mut max_transaction_id: u32 = 0;
        let mut events = Vec::new();
        let all = self.interface.db_get_all("transaction").await;
        for (key, value) in all.iter() {
            if key.starts_with("transaction_map:") {
                let payload = key.strip_prefix("transaction_map:").unwrap();
                let local_transaction_id: u32 = payload.parse().unwrap();
                let transaction_id: i32 = value.parse().unwrap();
                transaction_map.insert(local_transaction_id, transaction_id);
                unfinished_transactions.insert(local_transaction_id);
            }
        }
        for (key, value) in self.interface.db_get_all("transaction").await {
            if key.starts_with("event:") {
                let payload = key.strip_prefix("event:").unwrap();
                let index: u64 = payload.parse().unwrap();
                let event = serde_json::from_str::<TransactionEvent>(&value).unwrap();
                events.push(index);
                if let Some(local_transaction_id) = event.get_local_transaction_id() {
                    if !event.is_stop() {
                        unfinished_transactions.insert(local_transaction_id);
                    } else {
                        unfinished_transactions.remove(&local_transaction_id);
                    }
                }
            } else if key.starts_with("transaction_map:") {
                // already done
            } else if key.starts_with("transaction_connector_map:") {
                let payload = key.strip_prefix("transaction_connector_map:").unwrap();
                let local_transaction_id: u32 = payload.parse().unwrap();
                let connector_id: usize = value.parse().unwrap();
                transaction_connector_map.insert(local_transaction_id, connector_id);
            } else if key.starts_with("meter:") {
                let parts: Vec<&str> = key.strip_prefix("meter:").unwrap().split(':').collect();
                let local_transaction_id: u32 = parts[0].parse().unwrap();
                if let Some(v) = transaction_stop_meter_val_count.get_mut(&local_transaction_id) {
                    *v += 1;
                } else {
                    transaction_stop_meter_val_count.insert(local_transaction_id, 1);
                }
            } else if key.starts_with("num_transactions") {
                max_transaction_id = value.parse().unwrap();
            } else {
                unreachable!();
            }
        }
        let (tail, head) = events
            .first()
            .map(|x| *x)
            .zip(events.last().map(|x| x + 1))
            .unwrap_or((0, 0));
        (
            max_transaction_id,
            tail,
            head,
            transaction_map,
            transaction_connector_map,
            transaction_stop_meter_val_count,
            unfinished_transactions,
        )
    }
    pub async fn db_get_firmware_state(&mut self) -> FirmwareInstallStatus {
        let mut res = FirmwareInstallStatus::NA;
        if let Some(value) = self.interface.db_get("firmware", "state").await {
            let state = serde_json::from_str(&value).unwrap();
            res = state;
        }
        res
    }
    pub async fn db_update_config(&mut self, key: &str, value: &str) {
        self.interface
            .db_transaction("config", vec![(key, Some(value))])
            .await;
    }
    pub async fn db_get_from_cache(&mut self, id_tag: &str) -> Option<IdTagInfo> {
        self.interface
            .db_get("cache", &id_tag)
            .await
            .map(|s| serde_json::from_str(s).unwrap())
    }
    pub async fn db_update_cache(&mut self, id_tag: &str, info: IdTagInfo) {
        let value = serde_json::to_string(&info).unwrap();
        self.interface
            .db_transaction("cache", vec![(id_tag, Some(value.as_str()))])
            .await;
    }
    pub async fn db_clear_cache(&mut self) {
        self.interface.db_delete_table("cache").await;
    }
    pub async fn db_update_local_list(&mut self, version: i32, changes: Vec<LocalListChange>) {
        let mut ops: Vec<(String, Option<String>)> = changes
            .into_iter()
            .map(|f| match f {
                LocalListChange::Upsert { id_tag, info } => {
                    (id_tag, Some(serde_json::to_string(&info).unwrap()))
                }
                LocalListChange::Delete { id_tag } => (id_tag, None),
            })
            .collect();
        ops.push(("version#".to_string(), Some(version.to_string())));
        let ops_ref: Vec<(&str, Option<&str>)> = ops
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_deref()))
            .collect();
        self.interface.db_transaction("local_list", ops_ref).await;
    }
    pub async fn db_get_from_local_list(&mut self, id_tag: &str) -> Option<IdTagInfo> {
        self.interface
            .db_get("local_list", &id_tag)
            .await
            .map(|s| serde_json::from_str(s).unwrap())
    }
    pub async fn db_local_list_keys(&mut self) -> Vec<&str> {
        let v = self.interface.db_get_all("local_list").await;
        let mut res = Vec::new();
        for (key, _) in v {
            if key != "version#" {
                res.push(key);
            }
        }
        res
    }
    pub async fn db_get_local_list_version(&mut self) -> Option<i32> {
        self.interface
            .db_get("local_list", "version#")
            .await
            .map(|t| t.parse().unwrap())
    }
    pub async fn db_get_local_list_entries_count(&mut self) -> usize {
        let tot = self.interface.db_count_keys("local_list").await;
        if tot == 0 {
            return 0;
        }
        tot - 1
    }
    pub async fn db_change_firmware_state(&mut self, state: FirmwareInstallStatus) {
        let key = "state";
        let value = serde_json::to_string(&state).unwrap();
        self.interface
            .db_transaction("firmware", vec![(key, Some(value.as_str()))])
            .await;
    }
    pub(crate) async fn db_add_reservation(&mut self, reservation: ReserveNowRequest) {
        let key = reservation.reservation_id.to_string();
        let value = serde_json::to_string(&reservation).unwrap();
        self.interface
            .db_transaction("reservation", vec![(key.as_str(), Some(value.as_str()))])
            .await;
    }
    pub(crate) async fn db_remove_reservation(&mut self, reservation_id: i32) {
        self.interface
            .db_transaction(
                "reservation",
                vec![(reservation_id.to_string().as_str(), None)],
            )
            .await;
    }
    pub(crate) async fn db_push_transaction_event(&mut self, index: u64, event: TransactionEvent) {
        let key = format!("event:{}", index);
        let value = serde_json::to_string(&event).unwrap();
        let mut ops = Vec::new();
        ops.push((key, Some(value)));
        if let TransactionEvent::Start(t) = &event {
            ops.push((
                format!("transaction_connector_map:{}", t.local_transaction_id),
                Some(t.connector_id.to_string()),
            ));
            ops.push((
                "num_transactions".to_string(),
                Some(t.local_transaction_id.to_string()),
            ));
        }
        let ops_ref: Vec<(&str, Option<&str>)> = ops
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_deref()))
            .collect();
        self.interface.db_transaction("transaction", ops_ref).await;
    }
    pub(crate) async fn db_pop_transaction_event(
        &mut self,
        index: u64,
        local_transaction_id: Option<u32>,
        transaction_id: Option<i32>,
        meter_tx: Option<usize>,
    ) {
        let mut ops = Vec::new();
        ops.push((format!("event:{}", index), None));
        if let Some(local_transaction_id) = local_transaction_id {
            if let Some(transaction_id) = transaction_id {
                ops.push((
                    format!("transaction_map:{}", local_transaction_id),
                    Some(transaction_id.to_string()),
                ));
            }
            if let Some(meter_tx) = meter_tx {
                ops.push((format!("transaction_map:{}", local_transaction_id), None));
                ops.push((
                    format!("transaction_connector_map:{}", local_transaction_id),
                    None,
                ));
                for index in 0..meter_tx {
                    ops.push((format!("meter:{}:{}", local_transaction_id, index), None));
                }
            }
        }
        let ops_ref: Vec<(&str, Option<&str>)> = ops
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_deref()))
            .collect();
        self.interface.db_transaction("transaction", ops_ref).await;
    }

    pub(crate) async fn db_get_transaction_event(&mut self, index: u64) -> TransactionEvent {
        let val = self
            .interface
            .db_get("transaction", format!("event:{}", index).as_str())
            .await
            .unwrap();
        serde_json::from_str(val).unwrap()
    }

    pub(crate) async fn db_get_all_stop_meter_val(
        &mut self,
        local_transaction_id: u32,
        len: usize,
    ) -> Vec<MeterValueLocal> {
        let mut res = Vec::new();
        for index in 0..len {
            let val = self
                .interface
                .db_get(
                    "transaction",
                    format!("meter:{}:{}", local_transaction_id, index).as_str(),
                )
                .await
                .unwrap();
            let data = serde_json::from_str(val).unwrap();
            res.push(data);
        }
        res
    }
    pub(crate) async fn db_add_stop_meter_val(
        &mut self,
        local_transaction_id: u32,
        index: usize,
        value: MeterValueLocal,
    ) {
        let key = format!("meter:{}:{}", local_transaction_id, index);
        let value = serde_json::to_string(&value).unwrap();
        self.interface
            .db_transaction("transaction", vec![(key.as_str(), Some(value.as_str()))])
            .await;
    }
}
