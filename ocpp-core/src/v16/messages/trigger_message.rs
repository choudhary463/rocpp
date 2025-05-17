use super::super::types::{MessageTrigger, TriggerMessageStatus};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TriggerMessageRequest {
    pub requested_message: MessageTrigger,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_id: Option<usize>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TriggerMessageResponse {
    pub status: TriggerMessageStatus,
}
