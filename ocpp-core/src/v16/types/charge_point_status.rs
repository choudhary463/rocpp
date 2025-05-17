#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum ChargePointStatus {
    Available,
    Preparing,
    Charging,
    SuspendedEVSE,
    SuspendedEV,
    Finishing,
    Reserved,
    Unavailable,
    Faulted,
}
