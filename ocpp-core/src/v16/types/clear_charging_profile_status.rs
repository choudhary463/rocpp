#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ClearChargingProfileStatus {
    Accepted,
    Unknown,
}
