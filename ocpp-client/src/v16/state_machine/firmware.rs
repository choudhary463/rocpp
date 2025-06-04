use alloc::string::String;
use rocpp_core::v16::{
    messages::{
        firmware_status_notification::FirmwareStatusNotificationRequest,
        update_firmware::UpdateFirmwareRequest,
    },
    types::FirmwareStatus,
};

use crate::v16::{cp::ChargePoint, interfaces::ChargePointInterface};

use super::call::CallAction;

pub(crate) struct FirmwareDownloadInfo {
    pub retry_left: u64,
    pub retry_interval: u64,
    pub location: String,
}

pub(crate) enum FirmwareState {
    Idle,
    New(UpdateFirmwareRequest),
    Downloading(FirmwareDownloadInfo),
    DownloadSleep(FirmwareDownloadInfo),
    WaitingForTransactionToFinish,
    Installing,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) enum FirmwareInstallStatus {
    NA,
    InstallationSuccess,
    InstallationFailed,
}

impl FirmwareState {
    pub fn ongoing_firmware_update(&self) -> bool {
        matches!(self, FirmwareState::WaitingForTransactionToFinish | FirmwareState::Installing)
    }
}

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn on_firmware_online(&mut self) {
        let last_firmware_state = self.interface.db_get_firmware_state().await;
        let status = match last_firmware_state {
            FirmwareInstallStatus::InstallationSuccess => FirmwareStatus::Installed,
            FirmwareInstallStatus::InstallationFailed => FirmwareStatus::InstallationFailed,
            _ => return,
        };
        self.send_firmware_status_notification(status).await;
        self.interface.db_change_firmware_state(FirmwareInstallStatus::NA).await;
    }
    pub(crate) async fn send_firmware_status_notification(&mut self, status: FirmwareStatus) {
        let payload = FirmwareStatusNotificationRequest { status };
        self.enqueue_call(CallAction::FirmwareStatusNotification, payload).await;
    }
    pub(crate) async fn try_firmware_download(&mut self, info: FirmwareDownloadInfo) {
        if info.retry_left == 0 {
            self.send_firmware_status_notification(FirmwareStatus::DownloadFailed).await;
            self.firmware_state = FirmwareState::Idle;
        } else {
            self.download_firmware(info.location.clone()).await;
            self.firmware_state = FirmwareState::Downloading(info);
        }
    }
    pub(crate) async fn try_firmware_install(&mut self) {
        self.firmware_state = FirmwareState::Installing;
        self.send_firmware_status_notification(FirmwareStatus::Installing).await;
        self.install_firmware().await;
    }
    pub(crate) async fn trigger_firmware_status_notification(&mut self) {
        let status = match self.firmware_state {
            FirmwareState::Idle => FirmwareStatus::Idle,
            FirmwareState::New(_) => FirmwareStatus::Idle,
            FirmwareState::Downloading(_) => FirmwareStatus::Downloading,
            FirmwareState::DownloadSleep(_) => FirmwareStatus::Downloading,
            FirmwareState::WaitingForTransactionToFinish => FirmwareStatus::Downloaded,
            FirmwareState::Installing => FirmwareStatus::Installing,
        };
        self.send_firmware_status_notification(status).await;
    }
}
