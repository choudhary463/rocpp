use super::super::types::DiagnosticsStatus;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsStatusNotificationRequest {
    pub status: DiagnosticsStatus,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsStatusNotificationResponse {}
