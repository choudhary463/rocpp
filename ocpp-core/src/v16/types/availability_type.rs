#[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone, Debug)]
pub enum AvailabilityType {
    Inoperative,
    Operative,
}
