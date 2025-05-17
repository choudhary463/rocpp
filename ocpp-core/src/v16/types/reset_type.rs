#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum ResetType {
    Hard,
    Soft,
}
