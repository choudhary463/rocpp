#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct KeyValue {
    pub key: String,
    pub readonly: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}
