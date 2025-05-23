mod cp;
mod interface;
#[macro_use]
mod state_machine;
mod events;
mod services;

pub use {cp::config::ChargePointConfig, interface::{Database, Secc, TableOperation, HardwareActions, MeterData, MeterDataType, DiagnosticsResponse, TimerId, SeccState}};

#[cfg(feature = "async")]
pub use cp::r#async::ChargePointAsync as ChargePoint;

#[cfg(not(feature = "async"))]
pub use cp::core::ChargePointCore as ChargePoint;

#[cfg(feature = "async")]
pub use interface::{WebsocketIo, Firmware, Diagnostics, Timeout, StopChargePoint, Hardware};