use alloc::string::String;
use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDiagnosticsRequest {
    pub location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_time: Option<DateTime<Utc>>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDiagnosticsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}
