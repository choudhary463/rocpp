use alloc::{string::String, vec::Vec};
use core::task::{Context, Poll};

use chrono::{DateTime, Utc};
use rocpp_core::v16::types::{
    ChargePointErrorCode, ChargePointStatus, Location, Measurand, Phase, UnitOfMeasure,
};

#[allow(async_fn_in_trait)]
pub trait KeyValueStore {
    async fn db_init(&mut self);
    async fn db_transaction(&mut self, table: &str, ops: Vec<(&str, Option<&str>)>);
    async fn db_get(&mut self, table: &str, key: &str) -> Option<&str>;
    async fn db_get_all(&mut self, table: &str) -> Vec<(&str, &str)>;
    async fn db_count_keys(&mut self, table: &str) -> usize;
    async fn db_delete_table(&mut self, table: &str);
    async fn db_delete_all(&mut self);
}

//diagnostics

#[derive(Debug)]
pub enum DiagnosticsResponse {
    Timeout,
    Success,
    Failed,
}

#[allow(async_fn_in_trait)]
pub trait Diagnostics {
    async fn get_file_name(
        &mut self,
        start_time: Option<DateTime<Utc>>,
        stop_time: Option<DateTime<Utc>>,
    ) -> Option<String>;
    async fn diagnostics_upload(&mut self, location: String, timeout: u64);
    fn poll_diagnostics_upload(&mut self, cx: &mut Context<'_>) -> Poll<DiagnosticsResponse>;
}

// firmware

#[allow(async_fn_in_trait)]
pub trait Firmware {
    async fn firmware_download(&mut self, location: String);
    async fn firmware_install(&mut self);
    fn poll_firmware_download(&mut self, cx: &mut Context<'_>) -> Poll<bool>;
    fn poll_firmware_install(&mut self, cx: &mut Context<'_>) -> Poll<bool>;
}

// time

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

#[allow(async_fn_in_trait)]
pub trait TimeoutScheduler {
    async fn add_or_update_timeout(&mut self, id: TimerId, timeout: u64);
    async fn remove_timeout(&mut self, id: TimerId);
    async fn remove_all_timeouts(&mut self);
    fn poll_timeout(&mut self, cx: &mut Context<'_>) -> Poll<TimerId>;
}

// hardware
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
pub enum HardwareEvent {
    State(
        usize,
        SeccState,
        Option<ChargePointErrorCode>,
        Option<String>,
    ),
    IdTag(usize, String),
}

#[allow(async_fn_in_trait)]
pub trait Hardware {
    async fn get_boot_time(&self) -> u64;
    async fn hard_reset(&mut self);
    async fn update_status(&mut self, connector_id: usize, status: ChargePointStatus);
    async fn get_meter_value(
        &mut self,
        connector_id: usize,
        kind: &MeterDataType,
    ) -> Option<MeterData>;
    fn poll_hardware_events(&mut self, cx: &mut Context<'_>) -> Poll<HardwareEvent>;
    fn poll_reset(&mut self, cx: &mut Context<'_>) -> Poll<()>;
}

//ws

#[derive(Debug, PartialEq)]
pub enum WsEvent {
    Connected,
    Disconnected,
    Msg(String),
}

#[allow(async_fn_in_trait)]
pub trait Websocket {
    async fn ws_connect(&mut self, url: String);
    async fn ws_send(&mut self, msg: String);
    async fn ws_close(&mut self);
    fn poll_ws_recv(&mut self, cx: &mut Context<'_>) -> Poll<WsEvent>;
}

// main

pub trait ChargePointInterface:
    KeyValueStore + Diagnostics + Firmware + TimeoutScheduler + Hardware + Websocket
{
}

#[derive(Debug)]
pub enum ChargePointEvent {
    Reset,
    Hardware(HardwareEvent),
    Ws(WsEvent),
    Timeout(TimerId),
    FirmwareDownload(bool),
    FirmwareInstall(bool),
    Diagnostics(DiagnosticsResponse),
}
