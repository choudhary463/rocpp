#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum ReservationStatus {
    Accepted,
    Faulted,
    Occupied,
    Rejected,
    Unavailable,
}
