#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum ResetStatus {
    Accepted,
    Rejected,
}
