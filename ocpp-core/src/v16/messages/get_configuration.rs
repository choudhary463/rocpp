use super::super::types::KeyValue;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetConfigurationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<Vec<String>>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetConfigurationResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_key: Option<Vec<KeyValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_key: Option<Vec<String>>,
}
