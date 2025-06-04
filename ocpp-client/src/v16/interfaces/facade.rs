use alloc::{string::String, vec::Vec};
use core::task::{Context, Poll};

use chrono::{DateTime, Utc};
use rocpp_core::v16::types::ChargePointStatus;

use super::{ChargePointInterface, Diagnostics, DiagnosticsResponse, Firmware, Hardware, HardwareEvent, KeyValueStore, MeterData, MeterDataType, TimeoutScheduler, TimerId, Websocket, WsEvent};

pub struct ChargePointInterfaceFacade<K, D, Fw, Ts, Hw, Ws> {
    kv: K,
    diag: D,
    fw: Fw,
    ts: Ts,
    hw: Hw,
    ws: Ws,
}

impl<K, D, Fw, Ts, Hw, Ws> ChargePointInterfaceFacade<K, D, Fw, Ts, Hw, Ws> {
    pub fn new(kv: K, diag: D, fw: Fw, ts: Ts, hw: Hw, ws: Ws) -> Self {
        Self { kv, diag, fw, ts, hw, ws }
    }
}

impl<K, D, Fw, Ts, Hw, Ws> KeyValueStore for ChargePointInterfaceFacade<K, D, Fw, Ts, Hw, Ws>
where
    K: KeyValueStore,
{
    async fn db_init(&mut self) {
        self.kv.db_init().await
    }
    async fn db_transaction(&mut self, table: &str, ops: Vec<(&str, Option<&str>)>) {
        self.kv.db_transaction(table, ops).await
    }
    async fn db_get(&mut self, table: &str, key: &str) -> Option<&str> {
        self.kv.db_get(table, key).await
    }
    async fn db_get_all(&mut self, table: &str) -> Vec<(&str, &str)> {
        self.kv.db_get_all(table).await
    }
    async fn db_count_keys(&mut self, table: &str) -> usize {
        self.kv.db_count_keys(table).await
    }
    async fn db_delete_table(&mut self, table: &str) {
        self.kv.db_delete_table(table).await
    }
    async fn db_delete_all(&mut self) {
        self.kv.db_delete_all().await
    }
}

impl<K, D, Fw, Ts, Hw, Ws> Diagnostics for ChargePointInterfaceFacade<K, D, Fw, Ts, Hw, Ws>
where
    D: Diagnostics,
{
    async fn get_file_name(&mut self, start_time: Option<DateTime<Utc>>, stop_time: Option<DateTime<Utc>>) -> Option<String> {
        self.diag.get_file_name(start_time, stop_time).await
    }
    async fn diagnostics_upload(&mut self, location: String, timeout: u64) {
        self.diag.diagnostics_upload(location, timeout).await
    }
    fn poll_diagnostics_upload(&mut self, cx: &mut Context<'_>) -> Poll<DiagnosticsResponse> {
        self.diag.poll_diagnostics_upload(cx)
    }
}

impl<K, D, Fw, Ts, Hw, Ws> Firmware for ChargePointInterfaceFacade<K, D, Fw, Ts, Hw, Ws>
where
    Fw: Firmware,
{
    async fn firmware_download(&mut self, location: String) {
        self.fw.firmware_download(location).await
    }
    async fn firmware_install(&mut self) {
        self.fw.firmware_install().await
    }
    fn poll_firmware_download(&mut self, cx: &mut Context<'_>) -> Poll<bool> {
        self.fw.poll_firmware_download(cx)
    }
    fn poll_firmware_install(&mut self, cx: &mut Context<'_>) -> Poll<bool> {
        self.fw.poll_firmware_install(cx)
    }
}

impl<K, D, Fw, Ts, Hw, Ws> TimeoutScheduler for ChargePointInterfaceFacade<K, D, Fw, Ts, Hw, Ws>
where
    Ts: TimeoutScheduler,
{
    async fn add_or_update_timeout(&mut self, id: TimerId, timeout: u64) {
        self.ts.add_or_update_timeout(id, timeout).await
    }
    async fn remove_timeout(&mut self, id: TimerId) {
        self.ts.remove_timeout(id).await
    }
    async fn remove_all_timeouts(&mut self) {
        self.ts.remove_all_timeouts().await
    }
    fn poll_timeout(&mut self, cx: &mut Context<'_>) -> Poll<TimerId> {
        self.ts.poll_timeout(cx)
    }
}

impl<K, D, Fw, Ts, Hw, Ws> Hardware for ChargePointInterfaceFacade<K, D, Fw, Ts, Hw, Ws>
where
    Hw: Hardware,
{
    async fn get_boot_time(&self) -> u64 {
        self.hw.get_boot_time().await
    }
    async fn hard_reset(&mut self) {
        self.hw.hard_reset().await
    }
    async fn update_status(&mut self, connector_id: usize, status: ChargePointStatus) {
        self.hw.update_status(connector_id, status).await
    }
    async fn get_meter_value(&mut self, connector_id: usize, kind: &MeterDataType) -> Option<MeterData> {
        self.hw.get_meter_value(connector_id, kind).await
    }
    fn poll_hardware_events(&mut self, cx: &mut Context<'_>) -> Poll<HardwareEvent> {
        self.hw.poll_hardware_events(cx)
    }
    fn poll_reset(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        self.hw.poll_reset(cx)
    }
}

impl<K, D, Fw, Ts, Hw, Ws> Websocket for ChargePointInterfaceFacade<K, D, Fw, Ts, Hw, Ws>
where
    Ws: Websocket,
{
    async fn ws_connect(&mut self, url: String) {
        self.ws.ws_connect(url).await
    }
    async fn ws_send(&mut self, msg: String) {
        self.ws.ws_send(msg).await
    }
    async fn ws_close(&mut self) {
        self.ws.ws_close().await
    }
    fn poll_ws_recv(&mut self, cx: &mut Context<'_>) -> Poll<WsEvent> {
        self.ws.poll_ws_recv(cx)
    }
}

impl<K, D, Fw, Ts, Hw, Ws> ChargePointInterface for ChargePointInterfaceFacade<K, D, Fw, Ts, Hw, Ws>
where
    K: KeyValueStore,
    D: Diagnostics,
    Fw: Firmware,
    Ts: TimeoutScheduler,
    Hw: Hardware,
    Ws: Websocket,
{}