use alloc::vec::Vec;
use chrono::{DateTime, Utc};

use super::{ChargingRateUnitType, ChargingSchedulePeriod};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChargingSchedule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_schedule: Option<DateTime<Utc>>,
    pub charging_rate_unit: ChargingRateUnitType,
    pub charging_schedule_period: Vec<ChargingSchedulePeriod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_charging_rate: Option<f32>,
}
