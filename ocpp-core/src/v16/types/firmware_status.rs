#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum FirmwareStatus {
    Downloaded,
    DownloadFailed,
    Downloading,
    Idle,
    InstallationFailed,
    Installing,
    Installed,
}
