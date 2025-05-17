#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ChargingProfileStatus {
    Accepted,
    Rejected,
    NotSupported,
}
