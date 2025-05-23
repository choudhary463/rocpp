use alloc::string::String;

use super::IdTagInfo;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationData {
    pub id_tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_tag_info: Option<IdTagInfo>,
}
