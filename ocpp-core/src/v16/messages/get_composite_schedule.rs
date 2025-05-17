use chrono::{DateTime, Utc};

use super::super::types::{ChargingRateUnitType, ChargingSchedule, GetCompositeScheduleStatus};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCompositeScheduleRequest {
    pub connector_id: i32,
    pub duration: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_rate_unit: Option<ChargingRateUnitType>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCompositeScheduleResponse {
    pub status: GetCompositeScheduleStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_start: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_schedule: Option<ChargingSchedule>,
}
