#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ChargingProfilePurposeType {
    ChargePointMaxProfile,
    TxDefaultProfile,
    TxProfile,
}
