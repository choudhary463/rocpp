use alloc::string::String;
use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFirmwareRequest {
    pub location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u64>,
    pub retrieve_date: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_interval: Option<u64>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFirmwareResponse {}
