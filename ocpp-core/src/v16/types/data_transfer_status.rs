#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum DataTransferStatus {
    Accepted,
    Rejected,
    UnknownMessageId,
    UnknownVendorId,
}
