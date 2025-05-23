#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum UnlockStatus {
    Unlocked,
    UnlockFailed,
    NotSupported,
}
