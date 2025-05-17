use super::super::types::{ChargingProfilePurposeType, ClearChargingProfileStatus};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClearChargingProfileRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_profile_purpose: Option<ChargingProfilePurposeType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_level: Option<i32>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ClearChargingProfileResponse {
    pub status: ClearChargingProfileStatus,
}
