#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum RecurrencyKindType {
    Daily,
    Weekly,
}
