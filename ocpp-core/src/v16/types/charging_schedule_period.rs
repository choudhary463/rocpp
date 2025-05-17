#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChargingSchedulePeriod {
    pub start_period: i32,
    pub limit: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_phases: Option<i32>,
}
