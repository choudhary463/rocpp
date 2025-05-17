use super::super::types::{IdTagInfo, MeterValue, Reason};

use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StopTransactionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_tag: Option<String>,
    pub meter_stop: u64,
    pub timestamp: DateTime<Utc>,
    pub transaction_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<Reason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_data: Option<Vec<MeterValue>>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StopTransactionResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_tag_info: Option<IdTagInfo>,
}
