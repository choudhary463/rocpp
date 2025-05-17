use super::super::types::ClearCacheStatus;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ClearCacheRequest {}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClearCacheResponse {
    pub status: ClearCacheStatus,
}
