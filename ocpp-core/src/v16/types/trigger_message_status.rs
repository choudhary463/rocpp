#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum TriggerMessageStatus {
    Accepted,
    Rejected,
    NotImplemented,
}
