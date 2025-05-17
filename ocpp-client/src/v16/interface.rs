use std::task::{Context, Poll};

use chrono::{DateTime, Utc};
use ocpp_core::v16::types::{ChargePointStatus, Location, Measurand, Phase, UnitOfMeasure};

pub enum TableOperation {
    Insert { key: String, value: String },
    Delete { key: String },
}

impl TableOperation {
    pub fn insert(key: String, value: String) -> Self {
        TableOperation::Insert { key, value }
    }
    pub fn delete(key: String) -> Self {
        TableOperation::Delete { key }
    }
}

pub trait Database: Send + Unpin + 'static {
    fn init(&mut self);
    fn transaction(&mut self, table: &str, ops: Vec<TableOperation>);
    fn get(&mut self, table: &str, key: &str) -> Option<String>;
    fn get_all(&mut self, table: &str) -> Vec<(String, String)>;
    fn delete_table(&mut self, table: &str);
}

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

#[async_trait::async_trait]
pub trait Diagnostics: Send + Unpin + 'static {
    async fn upload(
        &mut self,
        location: String,
        file_name: String,
        start_time: Option<DateTime<Utc>>,
        stop_time: Option<DateTime<Utc>>,
    ) -> bool;
}

#[async_trait::async_trait]
pub trait Firmware: Send + Unpin + 'static {
    async fn download(&mut self, location: String) -> Option<Vec<u8>>;
    async fn install(&mut self, firmware_image: Vec<u8>) -> bool;
}

pub trait Secc: Send + 'static {
    fn hard_reset(&self);
    fn update_status(&self, connector_id: usize, status: ChargePointStatus);
    fn get_meter_value(&self, connector_id: usize, kind: &MeterDataType) -> Option<MeterData>;
}

#[async_trait::async_trait]
pub trait WebsocketIo: Send + Unpin + 'static {
    async fn connect(&mut self, url: String);
    fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<String>>;
    async fn send(&mut self, msg: String);
    async fn close(&mut self);
}
