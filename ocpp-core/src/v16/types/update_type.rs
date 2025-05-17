#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum UpdateType {
    Differential,
    Full,
}
