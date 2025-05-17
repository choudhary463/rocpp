use super::super::types::IdTagInfo;

use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartTransactionRequest {
    pub connector_id: usize,
    pub id_tag: String,
    pub meter_start: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservation_id: Option<i32>,
    pub timestamp: DateTime<Utc>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartTransactionResponse {
    pub id_tag_info: IdTagInfo,
    pub transaction_id: i32,
}
