#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum DiagnosticsStatus {
    Idle,
    Uploaded,
    UploadFailed,
    Uploading,
}
