#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum CancelReservationStatus {
    Accepted,
    Rejected,
}
