use alloc::{collections::{btree_map::BTreeMap, vec_deque::VecDeque}, format, string::{String, ToString}, vec, vec::Vec};
use ocpp_core::v16::{
    messages::{reserve_now::ReserveNowRequest, status_notification::StatusNotificationRequest},
    types::{AvailabilityType, ChargePointErrorCode, IdTagInfo},
};

use crate::v16::{
    interface::{Database, SeccState, TableOperation},
    state_machine::{
        auth::{CachedEntry, LocalListChange},
        connector::ConnectorState,
        firmware::FirmwareInstallStatus,
        transaction::{MeterValueLocal, TransactionEvent},
    },
};

pub struct DatabaseService<D> {
    db: D,
}

impl<D: Database> DatabaseService<D> {
    pub fn new(db: D) -> Self {
        Self { db }
    }
    pub(crate) fn db_update_config(&mut self, key: String, value: String) {
        self.db
            .transaction("config", vec![TableOperation::insert(key, value)]);
    }
    pub(crate) fn db_update_local_list(&mut self, version: i32, changes: Vec<LocalListChange>) {
        let mut ops: Vec<TableOperation> = changes
            .into_iter()
            .map(|f| match f {
                LocalListChange::Upsert { id_tag, info } => {
                    TableOperation::insert(id_tag, serde_json::to_string(&info).unwrap())
                }
                LocalListChange::Delete { id_tag } => TableOperation::delete(id_tag),
            })
            .collect();
        ops.push(TableOperation::insert(
            "version#".into(),
            version.to_string(),
        ));
        self.db.transaction("local_list", ops);
    }
    pub(crate) fn db_update_cache(&mut self, id_tag: String, info: CachedEntry) {
        let value = serde_json::to_string(&info).unwrap();
        self.db
            .transaction("cache", vec![TableOperation::insert(id_tag, value)]);
    }
    pub(crate) fn db_delete_cache(&mut self, id_tags: Vec<String>) {
        self.db.transaction(
            "cache",
            id_tags
                .into_iter()
                .map(TableOperation::delete)
                .collect(),
        );
    }
    pub(crate) fn db_remove_reservation(&mut self, reservation_id: i32) {
        self.db.transaction(
            "reservation",
            vec![TableOperation::delete(reservation_id.to_string())],
        );
    }
    pub(crate) fn db_push_transaction_event(&mut self, index: u64, event: TransactionEvent) {
        let key = format!("event:{}", index);
        let value = serde_json::to_string(&event).unwrap();
        let mut ops = Vec::new();
        ops.push(TableOperation::insert(key, value));
        if let TransactionEvent::Start(t) = &event {
            ops.push(TableOperation::insert(
                format!("transaction_connector_map:{}", t.local_transaction_id),
                t.connector_id.to_string(),
            ));
            ops.push(TableOperation::insert(
                "num_transactions".to_string(),
                t.local_transaction_id.to_string(),
            ));
        }
        self.db.transaction("transaction", ops);
    }
    pub(crate) fn db_pop_transaction_event(
        &mut self,
        index: u64,
        local_transaction_id: Option<u32>,
        transaction_id: Option<i32>,
        meter_tx: Option<usize>,
    ) {
        let mut ops = Vec::new();
        ops.push(TableOperation::delete(format!("event:{}", index)));
        if let Some(local_transaction_id) = local_transaction_id {
            if let Some(transaction_id) = transaction_id {
                ops.push(TableOperation::insert(
                    format!("transaction_map:{}", local_transaction_id),
                    transaction_id.to_string(),
                ));
            }
            if let Some(meter_tx) = meter_tx {
                ops.push(TableOperation::delete(format!(
                    "transaction_map:{}",
                    local_transaction_id
                )));
                ops.push(TableOperation::delete(format!(
                    "transaction_connector_map:{}",
                    local_transaction_id
                )));
                for index in 0..meter_tx {
                    ops.push(TableOperation::delete(format!(
                        "meter:{}:{}",
                        local_transaction_id, index
                    )));
                }
            }
        }
        self.db.transaction("transaction", ops);
    }
    pub(crate) fn db_add_meter_tx(
        &mut self,
        local_transaction_id: u32,
        index: usize,
        value: MeterValueLocal,
    ) {
        let key = format!("meter:{}:{}", local_transaction_id, index);
        let value = serde_json::to_string(&value).unwrap();
        self.db
            .transaction("transaction", vec![TableOperation::insert(key, value)]);
    }
    pub(crate) fn db_change_firmware_state(&mut self, state: FirmwareInstallStatus) {
        let key = "state".to_string();
        let value = serde_json::to_string(&state).unwrap();
        self.db
            .transaction("firmware", vec![TableOperation::insert(key, value)]);
    }
    pub(crate) fn db_change_operative_state(&mut self, connector_id: usize, state: AvailabilityType) {
        let key = connector_id.to_string();
        let value = serde_json::to_string(&state).unwrap();
        self.db
            .transaction("availabilitytype", vec![TableOperation::insert(key, value)]);
    }
    pub(crate) fn db_add_reservation(&mut self, reservation: ReserveNowRequest) {
        let key = reservation.reservation_id.to_string();
        let value = serde_json::to_string(&reservation).unwrap();
        self.db
            .transaction("reservation", vec![TableOperation::insert(key, value)]);
    }
    pub(crate) fn db_init(&mut self, mut default_configs: Vec<(String, String)>, clear_db: bool) {
        self.db.init();
        let mut previous_configs = self.db.get_all("previous_configs");

        previous_configs.sort();
        default_configs.sort();
        if clear_db || default_configs != previous_configs {
            self.db.delete_table("previous_configs");
            self.db.delete_table("config");
            self.db.delete_table("cache");
            self.db.delete_table("local_list");
            self.db.delete_table("availabilitytype");
            self.db.delete_table("firmware");
            self.db.delete_table("transaction");

            self.db.transaction(
                "previous_configs",
                default_configs
                    .clone()
                    .into_iter()
                    .map(|(key, value)| TableOperation::insert(key, value))
                    .collect(),
            );
            self.db.transaction(
                "config",
                default_configs
                    .into_iter()
                    .map(|(key, value)| TableOperation::insert(key, value))
                    .collect(),
            );
        }
    }
    pub(crate) fn get_all_config(&mut self) -> Vec<(String, String)> {
        self.db.get_all("config")
    }
    pub(crate) fn get_cache_data(&mut self) -> (BTreeMap<String, CachedEntry>, VecDeque<String>) {
        let mut db_cache_data: Vec<_> = self
            .db
            .get_all("cache")
            .into_iter()
            .map(|f| (f.0, serde_json::from_str::<CachedEntry>(&f.1).unwrap()))
            .collect();
        db_cache_data.sort_by_key(|(_, entry)| entry.updated_at);
        let mut cache = BTreeMap::new();
        let mut usage_order = VecDeque::new();
        for (tag, entry) in db_cache_data {
            cache.insert(tag.clone(), entry);
            usage_order.push_front(tag);
        }
        (cache, usage_order)
    }
    pub(crate) fn get_local_list(&mut self) -> (i32, BTreeMap<String, IdTagInfo>) {
        let mut version = 0;
        let mut res = BTreeMap::new();
        for (key, value) in self.db.get_all("local_list") {
            if key.as_str() == "version#" {
                version = value.parse().unwrap();
            } else {
                let info = serde_json::from_str(&value).unwrap();
                res.insert(key, info);
            }
        }
        (version, res)
    }
    fn get_reservations(&mut self) -> Vec<ReserveNowRequest> {
        self.db
            .get_all("reservation")
            .into_iter()
            .map(|f| serde_json::from_str::<ReserveNowRequest>(&f.1).unwrap())
            .collect()
    }
    fn get_operative_state(&mut self, num_connectors: usize) -> Vec<AvailabilityType> {
        let mut availability = vec![AvailabilityType::Operative; num_connectors];
        self.db
            .get_all("availabilitytype")
            .iter()
            .for_each(|(key, value)| {
                let connector_id: usize = key.parse().unwrap();
                let kind = serde_json::from_str::<AvailabilityType>(&value).unwrap();
                availability[connector_id] = kind;
            });
        availability
    }
    pub(crate) fn get_connector_state(
        &mut self,
        num_connectors: usize,
    ) -> (Vec<ConnectorState>, Vec<StatusNotificationRequest>) {
        let mut connector_state: Vec<_> = self
            .get_operative_state(num_connectors)
            .into_iter()
            .map(|f| match f {
                AvailabilityType::Operative => ConnectorState::Idle,
                AvailabilityType::Inoperative => ConnectorState::Unavailable(SeccState::Unplugged),
            })
            .collect();
        let reservations = self.get_reservations();
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
    pub(crate) fn get_firmware_state(&mut self) -> FirmwareInstallStatus {
        let mut res = FirmwareInstallStatus::NA;
        if let Some(value) = self.db.get("firmware", "state") {
            let state = serde_json::from_str(&value).unwrap();
            res = state;
        }
        res
    }
    pub fn get_transaction_data(
        &mut self,
    ) -> (
        u32,
        u64,
        u64,
        BTreeMap<u32, i32>,
        BTreeMap<u32, usize>,
        BTreeMap<u32, Vec<MeterValueLocal>>,
        VecDeque<TransactionEvent>,
    ) {
        let mut transaction_map: BTreeMap<u32, i32> = BTreeMap::new();
        let mut transaction_connector_map: BTreeMap<u32, usize> = BTreeMap::new();
        let mut stop_transaction_map_temp: BTreeMap<u32, Vec<(usize, MeterValueLocal)>> =
            BTreeMap::new();
        let mut max_transaction_id: u32 = 0;
        let mut events: Vec<(u64, TransactionEvent)> = Vec::new();
        for (key, value) in self.db.get_all("transaction") {
            if key.starts_with("event:") {
                let payload = key.strip_prefix("event:").unwrap();
                let index: u64 = payload.parse().unwrap();
                let event = serde_json::from_str::<TransactionEvent>(&value).unwrap();
                events.push((index, event));
            } else if key.starts_with("transaction_map:") {
                let payload = key.strip_prefix("transaction_map:").unwrap();
                let local_transaction_id: u32 = payload.parse().unwrap();
                let transaction_id: i32 = value.parse().unwrap();
                transaction_map.insert(local_transaction_id, transaction_id);
            } else if key.starts_with("transaction_connector_map:") {
                let payload = key.strip_prefix("transaction_connector_map:").unwrap();
                let local_transaction_id: u32 = payload.parse().unwrap();
                let connector_id: usize = value.parse().unwrap();
                transaction_connector_map.insert(local_transaction_id, connector_id);
            } else if key.starts_with("meter:") {
                let parts: Vec<&str> = key.strip_prefix("meter:").unwrap().split(':').collect();
                let local_transaction_id: u32 = parts[0].parse().unwrap();
                let index: usize = parts[1].parse().unwrap();
                let data = serde_json::from_str::<MeterValueLocal>(&value).unwrap();
                if let Some(v) = stop_transaction_map_temp.get_mut(&local_transaction_id) {
                    v.push((index, data));
                } else {
                    stop_transaction_map_temp.insert(local_transaction_id, vec![(index, data)]);
                }
            } else if key.starts_with("num_transactions") {
                max_transaction_id = value.parse().unwrap();
            } else {
                unreachable!();
            }
        }
        events.sort_by_key(|&(index, _)| index);
        assert!(events.windows(2).all(|w| w[1].0 == w[0].0 + 1));
        let (tail, head) = events
            .first()
            .map(|x| x.0)
            .zip(events.last().map(|x| x.0 + 1))
            .unwrap_or((0, 0));

        let mut stop_transaction_map: BTreeMap<u32, Vec<MeterValueLocal>> = BTreeMap::new();

        for (key, mut list) in stop_transaction_map_temp {
            list.sort_by_key(|&(index, _)| index);
            let is_continuous = list.windows(2).all(|w| w[1].0 == w[0].0 + 1);
            assert!((is_continuous && list.first().map(|x| x.0) == Some(0)));
            stop_transaction_map.insert(key, list.into_iter().map(|(_, v)| v).collect());
        }

        (
            max_transaction_id,
            tail,
            head,
            transaction_map,
            transaction_connector_map,
            stop_transaction_map,
            events.into_iter().map(|f| f.1).collect(),
        )
    }
}
