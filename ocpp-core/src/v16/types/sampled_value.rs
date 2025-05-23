use alloc::string::String;

use super::{value_format::ValueFormat, Location, Measurand, Phase, ReadingContext, UnitOfMeasure};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SampledValue {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ReadingContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ValueFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measurand: Option<Measurand>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<Phase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<UnitOfMeasure>,
}
