use super::super::types::ReservationStatus;
use alloc::string::String;
use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReserveNowRequest {
    pub connector_id: usize,
    pub expiry_date: DateTime<Utc>,
    pub id_tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id_tag: Option<String>,
    pub reservation_id: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReserveNowResponse {
    pub status: ReservationStatus,
}
