#[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone, Debug)]
pub enum RegistrationStatus {
    Accepted,
    Pending,
    Rejected,
}
