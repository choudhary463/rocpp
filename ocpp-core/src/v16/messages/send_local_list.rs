use alloc::vec::Vec;

use super::super::types::{AuthorizationData, UpdateStatus, UpdateType};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SendLocalListRequest {
    pub list_version: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_authorization_list: Option<Vec<AuthorizationData>>,
    pub update_type: UpdateType,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SendLocalListResponse {
    pub status: UpdateStatus,
}
