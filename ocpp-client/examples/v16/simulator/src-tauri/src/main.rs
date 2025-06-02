// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use flume::unbounded;
use interface::{
    database::DatabaseService,
    firmware::FirmwareService,
    log::init_log,
    hardware::HardwareService,
    ui::{run_ui, UiClient}
};
use log::LevelFilter;
use ocpp_client::v16::{ChargePointAsync, ChargePointConfig, FlumePeripheral, FtpDiagnostics, FtpFirmwareDownload, PeripheralActions, TokioShutdown, TokioTimerManager, TokioWsClient};
use tokio_util::sync::CancellationToken;

mod interface;

#[tokio::main]
async fn main() {
    let log_level = LevelFilter::Debug;

    let mut configs: ChargePointConfig = {
        let raw = std::fs::read_to_string("config.json").expect("missing config.json");
        serde_json::from_str(&raw).expect("invalid config format")
    };
    configs.seed = rand::random();
    let num_connectors: usize = configs
        .default_ocpp_configs
        .iter()
        .find(|&t| t.0 == "NumberOfConnectors")
        .unwrap()
        .1
        .parse()
        .unwrap();
    {
        let mut db = DatabaseService::new("simulator.db");
        configs.boot_info.firmware_version = Some(db.get_firmware_version());
    }
    let (ui_tx, ui_rx) = unbounded();
    let ui = UiClient::new(ui_tx);

    init_log(ui.clone(), log_level);

    let ui_clone = ui.clone();
    let (peripheral_tx, peripheral_rx) = unbounded::<PeripheralActions>();
    let peripheral_tx_clone = peripheral_tx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3)).await;
        ui_clone.init(num_connectors);
        loop {
            let db = DatabaseService::new("simulator.db");
            let diagnostics = FtpDiagnostics::new();
            let fw_download = FtpFirmwareDownload::new();
            let firmware = FirmwareService::new(db.clone(), fw_download);
            let stop_token = CancellationToken::new();
            let hw = HardwareService::new(ui.clone(), stop_token.clone());
            let timer = TokioTimerManager::new();
            let peripheral = FlumePeripheral::from_channel(peripheral_tx.clone(), peripheral_rx.clone());
            let shutdown = TokioShutdown::from_token(stop_token.clone());
            let ws = TokioWsClient::new();
            let _ = peripheral_rx.drain();
            log::info!("ChargePoint Started");
            let cp = ChargePointAsync::new(
                ws,
                diagnostics,
                firmware,
                db,
                hw,
                timer,
                peripheral,
                shutdown,
                configs.clone()
            );
            ui_clone.update_charger_state(true);
            tokio::select! {
                biased;
                _ = stop_token.cancelled() => {
                    log::debug!("ChargePoint stop_token cancelled")
                }
                _ = cp.run() => {

                }
            }
            ui_clone.update_charger_state(false);
            log::info!("ChargePoint exited, restarting...");
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
    run_ui(peripheral_tx_clone, ui_rx).await;
}
