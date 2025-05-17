use super::super::types::ConfigurationStatus;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ChangeConfigurationRequest {
    pub key: String,
    pub value: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ChangeConfigurationResponse {
    pub status: ConfigurationStatus,
}
