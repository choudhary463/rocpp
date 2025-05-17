#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum GetCompositeScheduleStatus {
    Accepted,
    Rejected,
}
