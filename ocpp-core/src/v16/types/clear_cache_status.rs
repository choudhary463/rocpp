#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum ClearCacheStatus {
    Accepted,
    Rejected,
}
