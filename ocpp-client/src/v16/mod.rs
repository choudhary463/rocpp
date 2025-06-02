mod cp;
mod drivers;
#[macro_use]
mod state_machine;
mod events;

pub use {cp::config::ChargePointConfig, drivers::{database::{Database, TableOperation}, hardware_interface::{HardwareInterface, MeterData, MeterDataType}, diagnostics::DiagnosticsResponse, peripheral_input::{SeccState, PeripheralActions}, timers::TimerId}};

#[cfg(feature = "async")]
pub use cp::r#async::ChargePointAsync;

#[cfg(not(feature = "async"))]
pub use cp::core::ChargePointCore;

#[cfg(feature = "async")]
pub use {drivers::{diagnostics::Diagnostics, firmware::{FirmwareDownload, FirmwareInstall, Firmware}, peripheral_input::PeripheralInput, shutdown::ShutdownSignal, timers::TimerManager, websocket::WebsocketTransport}};

#[cfg(feature = "flume_peripheral")]
pub use drivers::peripheral_input::FlumePeripheral;

#[cfg(feature = "tokio_timer")]
pub use drivers::timers::TokioTimerManager;

#[cfg(feature = "tokio_shutdown")]
pub use drivers::shutdown::TokioShutdown;

#[cfg(feature = "ftp_transfer")]
pub use drivers::{diagnostics::FtpDiagnostics, firmware::FtpFirmwareDownload};

#[cfg(feature = "tokio_ws")]
pub use drivers::websocket::TokioWsClient;