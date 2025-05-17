use super::super::types::{ChargingProfile, ChargingProfileStatus};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetChargingProfileRequest {
    pub connector_id: i32,
    #[serde(rename = "csChargingProfiles")]
    pub cs_charging_profiles: ChargingProfile,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetChargingProfileResponse {
    pub status: ChargingProfileStatus,
}
