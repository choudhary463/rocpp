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
use rocpp_client::v16::{ChargePoint, ChargePointConfig, ChargePointInterfaceFacade, HardwareEvent};
use tokio_util::sync::CancellationToken;

use crate::interface::{diagnostics::DiagnosticsService, timers::TokioTimerServie, ws::WsClient};

mod interface;

#[tokio::main]
async fn main() {
    let log_level = LevelFilter::Debug;
    let db_path = std::env::temp_dir().join("config.json");

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
        let mut db = DatabaseService::new(db_path.clone());
        configs.boot_info.firmware_version = Some(db.get_firmware_version().await);
    }
    let (ui_tx, ui_rx) = unbounded();
    let ui = UiClient::new(ui_tx);

    init_log(ui.clone(), log_level);

    let ui_clone = ui.clone();
    let (hardware_tx, hardware_rx) = unbounded::<HardwareEvent>();
    let hardware_tx_clone = hardware_tx.clone();
    let local_handle = std::thread::spawn({
        move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to build current_thread runtime");

            rt.block_on(async move {
                let local_set = tokio::task::LocalSet::new();
                local_set.spawn_local(async move {
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    ui_clone.init(num_connectors);

                    loop {
                        let db = DatabaseService::new(db_path.clone());
                        let diagnostics = DiagnosticsService::new();
                        let firmware = FirmwareService::new(db.clone());
                        let stop_token = CancellationToken::new();
                        let hardware = HardwareService::new(
                            ui.clone(),
                            stop_token.clone(),
                            hardware_rx.clone(),
                        );
                        let timer = TokioTimerServie::new();
                        let ws = WsClient::new();
                        let _ = hardware_rx.drain();

                        let interface = ChargePointInterfaceFacade::new(
                            db.clone(),
                            diagnostics,
                            firmware,
                            timer,
                            hardware,
                            ws,
                        );
                        log::info!("ChargePoint Started");
                        ui.update_charger_state(true);
                        if let Err(e) = tokio::task::spawn_local(
                            ChargePoint::run(interface, configs.clone()),
                        )
                        .await
                        {
                            log::error!("ChargePoint crashed: {}", e);
                        }

                        ui.update_charger_state(false);
                        log::info!("ChargePoint exited, restarting...");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                });
                local_set.await;
            });
        }
    });
    run_ui(hardware_tx_clone, ui_rx).await;
    let _ = local_handle.join();
}
