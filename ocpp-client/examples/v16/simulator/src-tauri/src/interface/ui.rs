use flume::{unbounded, Receiver, Sender};
use ocpp_client::v16::{PeripheralActions, SeccState};
use ocpp_core::v16::types::ChargePointStatus;
use serde_json::{json, Value};
use tauri::Manager;

#[derive(serde::Serialize)]
pub enum UiRequest {
    Init {
        num_connectors: usize,
    },
    ConnectorStatus {
        connector_id: usize,
        status: ChargePointStatus,
    },
    ChargePointStatus {
        running: bool,
    },
    Log {
        kind: String,
        message: String,
    },
}

#[derive(Clone, Debug)]
pub struct UiClient {
    tx: Sender<UiRequest>,
}

impl UiClient {
    pub fn new(tx: Sender<UiRequest>) -> Self {
        Self { tx }
    }
    pub fn init(&self, num_connectors: usize) {
        let _ = self.tx.send(UiRequest::Init { num_connectors });
    }
    pub fn update_connector_state(&self, connector_id: usize, status: ChargePointStatus) {
        let _ = self.tx.send(UiRequest::ConnectorStatus {
            connector_id,
            status,
        });
    }
    pub fn update_charger_state(&self, running: bool) {
        let _ = self.tx.send(UiRequest::ChargePointStatus { running });
    }
    pub fn log(&self, kind: String, message: String) {
        let _ = self.tx.send(UiRequest::Log { kind, message });
    }
}

pub struct TauriState {
    pub secc_tx: Sender<PeripheralActions>,
}

#[tauri::command]
pub fn send_id_tag(connector_id: usize, id_tag: String, state: tauri::State<'_, TauriState>) {
    let _ = state
        .secc_tx
        .send(PeripheralActions::IdTag(connector_id - 1, id_tag));
}

#[tauri::command]
pub fn set_connector_state(
    connector_id: usize,
    state_str: String,
    state: tauri::State<'_, TauriState>,
) {
    let secc_state = match state_str.as_str() {
        "plug" => SeccState::Plugged,
        "unplug" => SeccState::Unplugged,
        "faulty" => SeccState::Faulty,
        _ => return,
    };

    let _ = state
        .secc_tx
        .send(PeripheralActions::State(connector_id - 1, secc_state, None, None));
}

pub async fn run_ui(secc_tx: Sender<PeripheralActions>, req_rx: Receiver<UiRequest>) {
    let (ui_tx, ui_rx) = unbounded();
    tokio::spawn(async move { handle_ui_req(ui_tx, req_rx).await });
    tauri::Builder::default()
        .manage(TauriState { secc_tx })
        .setup(|app| {
            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                loop {
                    let req = ui_rx.recv_async().await.unwrap();
                    app_handle.emit_all(&req.0, req.1).unwrap();
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![send_id_tag, set_connector_state])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn handle_ui_req(ui_tx: Sender<(String, Value)>, req_rx: Receiver<UiRequest>) {
    loop {
        let req = req_rx.recv_async().await.unwrap();
        match req {
            UiRequest::ConnectorStatus {
                connector_id,
                status,
            } => {
                let _ = ui_tx
                    .send_async((
                        format!("connector_status"),
                        json!({"connector_id": connector_id, "status": status}),
                    ))
                    .await;
            }
            UiRequest::ChargePointStatus { running } => {
                let _ = ui_tx
                    .send_async((format!("charger_status"), json!({"running": running})))
                    .await;
            }
            UiRequest::Log { kind, message } => {
                let _ = ui_tx
                    .send_async((format!("log"), json!({"kind": kind, "message": message})))
                    .await;
            }
            UiRequest::Init { num_connectors } => {
                let _ = ui_tx
                    .send_async((format!("init"), json!({"connectors": num_connectors})))
                    .await;
            }
        }
    }
}
