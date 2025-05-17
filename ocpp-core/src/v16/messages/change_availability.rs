use super::super::types::{AvailabilityStatus, AvailabilityType};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangeAvailabilityRequest {
    pub connector_id: usize,
    #[serde(rename = "type")]
    pub kind: AvailabilityType,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ChangeAvailabilityResponse {
    pub status: AvailabilityStatus,
}
