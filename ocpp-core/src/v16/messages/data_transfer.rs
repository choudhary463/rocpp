use alloc::string::String;

use super::super::types::DataTransferStatus;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DataTransferRequest {
    pub vendor_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DataTransferResponse {
    pub status: DataTransferStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}
