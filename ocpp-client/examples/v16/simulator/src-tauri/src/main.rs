// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use flume::unbounded;
use interface::{
    database::DatabaseService,
    diagnostics::DiagnosticsService,
    firmware::FirmwareService,
    log::init_log,
    secc::SeccService,
    ui::{run_ui, UiClient},
    websocket::WsService,
};
use log::LevelFilter;
use ocpp_client::v16::{ChargePoint, ChargePointConfig};
use tokio_util::sync::CancellationToken;

mod interface;

#[tokio::main]
async fn main() {
    let log_level = LevelFilter::Debug;

    let mut config: ChargePointConfig = {
        let raw = std::fs::read_to_string("config.json").expect("missing config.json");
        serde_json::from_str(&raw).expect("invalid config format")
    };
    let num_connectors: usize = config
        .default_ocpp_configs
        .get("NumberOfConnectors")
        .unwrap()
        .parse()
        .unwrap();
    {
        let mut db = DatabaseService::new("simulator.db");
        config.boot_info.firmware_version = Some(db.get_firmware_version());
    }
    let (ui_tx, ui_rx) = unbounded();
    let ui = UiClient::new(ui_tx);

    init_log(ui.clone(), log_level);

    let ui_clone = ui.clone();
    let (secc_tx, secc_rx) = unbounded();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3)).await;
        ui_clone.init(num_connectors);
        loop {
            let db = DatabaseService::new("simulator.db");
            let diagnostics = DiagnosticsService::new();
            let firmware = FirmwareService::new(db.clone());
            let stop_token = CancellationToken::new();
            let secc = SeccService::new(ui.clone(), stop_token.clone());
            let ws = WsService::new();
            let _ = secc_rx.drain();
            log::info!("ChargePoint Started");
            let cp = ChargePoint::new(
                ws,
                diagnostics,
                firmware,
                db,
                secc,
                config.clone(),
                stop_token.clone(),
                secc_rx.clone(),
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
    run_ui(secc_tx, ui_rx).await;
}
