#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum UnitOfMeasure {
    Wh,
    #[serde(rename = "kWh")]
    KWh,
    #[serde(rename = "varh")]
    Varh,
    #[serde(rename = "kvarh")]
    Kvarh,
    W,
    #[serde(rename = "kW")]
    Kw,
    #[serde(rename = "VA")]
    Va,
    #[serde(rename = "kVA")]
    Kva,
    #[serde(rename = "var")]
    Var,
    #[serde(rename = "kvar")]
    Kvar,
    A,
    V,
    Celsius,
    Fahrenheit,
    K,
    Percent,
}
