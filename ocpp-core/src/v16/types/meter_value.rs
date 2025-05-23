use alloc::vec::Vec;
use chrono::DateTime;
use chrono::Utc;

use super::SampledValue;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MeterValue {
    pub timestamp: DateTime<Utc>,
    pub sampled_value: Vec<SampledValue>,
}
