#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum AuthorizationStatus {
    Accepted,
    Blocked,
    Expired,
    Invalid,
    ConcurrentTx,
}
