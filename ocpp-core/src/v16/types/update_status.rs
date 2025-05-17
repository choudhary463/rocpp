#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum UpdateStatus {
    Accepted,
    Failed,
    NotSupported,
    VersionMismatch,
}
