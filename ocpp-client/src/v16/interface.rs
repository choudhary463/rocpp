#[cfg(feature = "async")]
use core::task::{Context, Poll};

use alloc::{string::String, vec::Vec};
#[cfg(feature = "async")]
use chrono::{DateTime, Utc};
use ocpp_core::v16::types::{ChargePointErrorCode, ChargePointStatus, Location, Measurand, Phase, UnitOfMeasure};

#[cfg(feature = "async")]
use alloc::boxed::Box;

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

#[derive(Clone, PartialEq, Debug)]
pub enum SeccState {
    Plugged,
    Unplugged,
    Faulty,
}

#[derive(Debug)]
pub enum HardwareActions {
    State(
        usize,
        SeccState,
        Option<ChargePointErrorCode>,
        Option<String>,
    ),
    IdTag(usize, String),
}

#[derive(Debug)]
pub enum DiagnosticsResponse {
    Timeout,
    Success,
    Failed
}

#[derive(Eq, Hash, Clone, Copy, PartialEq, Debug)]
pub enum TimerId {
    Boot,
    Heartbeat,
    Call,
    StatusNotification(usize),
    Transaction,
    Authorize(usize),
    Reservation(usize),
    Firmware,
    MeterAligned,
    MeterSampled(usize),
}

pub trait Database: Send + Unpin + 'static {
    fn init(&mut self);
    fn transaction(&mut self, table: &str, ops: Vec<TableOperation>);
    fn get(&mut self, table: &str, key: &str) -> Option<String>;
    fn get_all(&mut self, table: &str) -> Vec<(String, String)>;
    fn delete_table(&mut self, table: &str);
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait Diagnostics: Send + Unpin + 'static {
    async fn upload(
        &mut self,
        location: String,
        file_name: String,
        start_time: Option<DateTime<Utc>>,
        stop_time: Option<DateTime<Utc>>,
        timeout: u64
    ) -> DiagnosticsResponse;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait Firmware: Send + Unpin + 'static {
    async fn download(&mut self, location: String) -> Option<Vec<u8>>;
    async fn install(&mut self, firmware_image: Vec<u8>) -> bool;
}

pub trait Secc: Send + 'static {
    fn get_boot_time(&self) -> u128;
    fn hard_reset(&self);
    fn update_status(&self, connector_id: usize, status: ChargePointStatus);
    fn get_meter_value(&self, connector_id: usize, kind: &MeterDataType) -> Option<MeterData>;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait WebsocketIo: Send + Unpin + 'static {
    async fn connect(&mut self, url: String);
    fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<String>>;
    async fn send(&mut self, msg: String);
    async fn close(&mut self);
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait Hardware: Send + Unpin + 'static {
    fn poll_next_action(&mut self, cx: &mut Context<'_>) -> Poll<HardwareActions>;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait Timeout: Send + Unpin + 'static {
    fn add_or_update_timeout(&mut self, id: TimerId, timeout: u64);
    fn remove_timeout(&mut self, id: TimerId);
    fn remove_all_timeouts(&mut self);
    fn poll_timeout(&mut self, cx: &mut Context<'_>) -> Poll<TimerId>;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait StopChargePoint: Send + Unpin + 'static {
    fn poll_stopped(&mut self, cx: &mut Context<'_>) -> Poll<()>;
}
