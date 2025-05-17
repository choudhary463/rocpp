#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum Reason {
    DeAuthorized,
    EmergencyStop,
    EVDisconnected,
    HardReset,
    Local,
    Other,
    PowerLoss,
    Reboot,
    Remote,
    SoftReset,
    UnlockCommand,
}
