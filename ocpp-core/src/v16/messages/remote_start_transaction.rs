use super::super::types::{ChargingProfile, RemoteStartStopStatus};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoteStartTransactionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_id: Option<usize>,
    pub id_tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_profile: Option<ChargingProfile>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoteStartTransactionResponse {
    pub status: RemoteStartStopStatus,
}
