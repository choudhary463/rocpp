#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum RemoteStartStopStatus {
    Accepted,
    Rejected,
}
