#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum ConfigurationStatus {
    Accepted,
    Rejected,
    RebootRequired,
    NotSupported,
}
