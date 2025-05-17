#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum Location {
    Body,
    Cable,
    #[serde(rename = "EV")]
    Ev,
    Inlet,
    Outlet,
}
