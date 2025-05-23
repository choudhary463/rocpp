use super::super::types::RegistrationStatus;
use alloc::string::String;
use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BootNotificationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_box_serial_number: Option<String>,
    pub charge_point_model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_point_serial_number: Option<String>,
    pub charge_point_vendor: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iccid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imsi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meter_serial_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meter_type: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BootNotificationResponse {
    pub current_time: DateTime<Utc>,
    pub interval: u64,
    pub status: RegistrationStatus,
}
