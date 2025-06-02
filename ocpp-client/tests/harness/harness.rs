use std::{path::PathBuf, sync::Once};

use flume::Sender;
use log::LevelFilter;
use ocpp_client::v16::{ChargePointAsync, ChargePointConfig, Database, FlumePeripheral, PeripheralActions, TokioShutdown, TokioTimerManager};
use ocpp_core::v16::messages::boot_notification::BootNotificationRequest;
use tokio_util::sync::CancellationToken;

use crate::harness::event::{Event, SeccEvents};

use super::{
    database::{FileDatabase, MockDatabase}, diagnostics::MockDiagnostics, event::{event_bus, EventRx}, firmware::MockFirmware, hardware::MockHardware, ws::{MockWs, MockWsHandle}
};

#[derive(Debug)]
pub struct CpHarness {
    pub ws_handle: MockWsHandle,
    pub bus_rx: EventRx,
    pub peripheral_tx: Sender<PeripheralActions>,
    pub stop_token: CancellationToken,
}

fn default_ocpp_configs() -> Vec<(String, String)> {
    let configs = vec![
        ("HeartbeatInterval", "10"),
        ("MinimumStatusDuration", "0"),
        ("AuthorizationCacheEnabled", "false"),
        ("LocalAuthListEnabled", "false"),
        ("LocalAuthListMaxLength", "1000"),
        ("SendLocalListMaxLength", "1000"),
        ("AllowOfflineTxForUnknownId", "false"),
        ("LocalAuthorizeOffline", "false"),
        ("LocalPreAuthorize", "false"),
        ("NumberOfConnectors", "2"),
        ("ConnectionTimeOut", "4"),
        ("StopTransactionOnEVSideDisconnect", "false"),
        ("MeterValueSampleInterval", "0"),
        ("ClockAlignedDataInterval", "0"),
        ("MeterValuesSampledData", "Energy.Active.Import.Register"),
        ("StopTxnSampledData", "Energy.Active.Import.Register"),
        ("MeterValuesAlignedData", "Energy.Active.Import.Register"),
        ("StopTxnAlignedData", "Energy.Active.Import.Register"),
        ("MeterValuesSampledDataMaxLength", "10"),
        ("StopTxnSampledDataMaxLength", "10"),
        ("MeterValuesAlignedDataMaxLength", "10"),
        ("StopTxnAlignedDataMaxLength", "10"),
        ("StopTransactionOnInvalidId", "false"),
        ("TransactionMessageAttempts", "3"),
        ("TransactionMessageRetryInterval", "2"),
        ("AuthorizeRemoteTxRequests", "false"),
        ("ConnectorPhaseRotation", ""),
        ("ResetRetries", "1"),
        ("GetConfigurationMaxKeys", "10"),
        (
            "SupportedFeatureProfiles",
            "Core,FirmwareManagement,LocalAuthListManagement,Reservation,RemoteTrigger",
        ),
        ("UnlockConnectorOnEVSideDisconnect", "false"),
    ];
    configs
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn get_call_timeout() -> u64 {
    5
}

fn get_max_cache_len() -> usize {
    100
}

pub fn get_cms_url() -> String {
    String::from("temp")
}

fn get_boot_info() -> BootNotificationRequest {
    BootNotificationRequest {
        charge_box_serial_number: None,
        charge_point_model: format!("CP"),
        charge_point_serial_number: None,
        charge_point_vendor: format!("IDK"),
        firmware_version: None,
        iccid: None,
        imsi: None,
        meter_serial_number: None,
        meter_type: None,
    }
}

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Trace)
            .try_init()
            .ok();
    });
}

impl CpHarness {
    pub fn new_helper<D: Database>(
        test_timeout: u64,
        override_defualt_configs: Vec<(&str, &str)>,
        db: D,
        clear_db: bool,
    ) -> Self {
        let (tx, rx) = event_bus(test_timeout);
        let (ws, ws_handle) = MockWs::new(tx.clone());
        let diagnostics = MockDiagnostics {};
        let firmware = MockFirmware {};
        let shutdown = TokioShutdown::new();
        let peripheral = FlumePeripheral::new();
        let timer = TokioTimerManager::new();
        let hw = MockHardware::new(shutdown.get_token());
        let mut default_ocpp_configs = default_ocpp_configs();
        for (key, value) in override_defualt_configs {
            if let Some(config) = default_ocpp_configs.iter_mut().find(|x| x.0 == key) {
                config.1 = value.to_string()
            } else {
                default_ocpp_configs.push((key.to_string(), value.to_string()));
            }
        }
        let configs = ChargePointConfig {
            cms_url: get_cms_url(),
            call_timeout: get_call_timeout(),
            max_cache_len: get_max_cache_len(),
            boot_info: get_boot_info(),
            default_ocpp_configs,
            clear_db,
            seed: rand::random()
        };
        let peripheral_tx = peripheral.get_sender();
        let stop_token = shutdown.get_token();
        let stop_token_clone = shutdown.get_token();
        let cp = ChargePointAsync::new(
            ws,
            diagnostics,
            firmware,
            db,
            hw,
            timer,
            peripheral,
            shutdown,
            configs
        );
        tokio::spawn(async move {
            tokio::select! {
                biased;
                _ = stop_token_clone.cancelled() => {
                    tx.push(Event::Secc(SeccEvents::HardReset));
                    return;
                }
                _ = tokio::spawn(cp.run()) => {
                    tx.push(Event::Secc(SeccEvents::Crashed));
                }
            }
        });
        Self {
            ws_handle,
            bus_rx: rx,
            peripheral_tx,
            stop_token,
        }
    }
    pub fn new(
        timeout: u64,
        override_defualt_configs: Vec<(&str, &str)>,
        db_dir: Option<PathBuf>,
        clear_db: bool,
    ) -> Self {
        init_logger();
        if let Some(dir) = db_dir {
            Self::new_helper(
                timeout,
                override_defualt_configs,
                FileDatabase::new(dir),
                clear_db,
            )
        } else {
            Self::new_helper(
                timeout,
                override_defualt_configs,
                MockDatabase::new(),
                clear_db,
            )
        }
    }
}
