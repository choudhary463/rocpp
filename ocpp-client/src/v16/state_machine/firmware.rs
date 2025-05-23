use alloc::{string::String, vec::Vec};
use ocpp_core::v16::{
    messages::{
        firmware_status_notification::FirmwareStatusNotificationRequest,
        update_firmware::UpdateFirmwareRequest,
    },
    types::FirmwareStatus,
};

use crate::v16::{interface::{Database, Secc}, cp::ChargePointCore};

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
    WaitingForTransactionToFinish(Vec<u8>),
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
        matches!(self, FirmwareState::WaitingForTransactionToFinish(_) | FirmwareState::Installing)
    }
}

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub(crate) fn on_firmware_online(&mut self) {
        let status = match self.last_firmware_state {
            FirmwareInstallStatus::InstallationSuccess => FirmwareStatus::Installed,
            FirmwareInstallStatus::InstallationFailed => FirmwareStatus::InstallationFailed,
            _ => return,
        };
        self.send_firmware_status_notification(status);
        self.last_firmware_state = FirmwareInstallStatus::NA;
        self.db.db_change_firmware_state(FirmwareInstallStatus::NA);
    }
    pub(crate) fn send_firmware_status_notification(&mut self, status: FirmwareStatus) {
        let payload = FirmwareStatusNotificationRequest { status };
        self.enqueue_call(CallAction::FirmwareStatusNotification, payload);
    }
    pub(crate) fn try_firmware_download(&mut self, info: FirmwareDownloadInfo) {
        if info.retry_left == 0 {
            self.send_firmware_status_notification(FirmwareStatus::DownloadFailed);
            self.firmware_state = FirmwareState::Idle;
        } else {
            self.download_firmware(info.location.clone());
            self.firmware_state = FirmwareState::Downloading(info);
        }
    }
    pub(crate) fn try_firmware_install(&mut self, firmware_image: Vec<u8>) {
        self.firmware_state = FirmwareState::Installing;
        self.send_firmware_status_notification(FirmwareStatus::Installing);
        self.install_firmware(firmware_image);
    }
    pub(crate) fn trigger_firmware_status_notification(&mut self) {
        let status = match self.firmware_state {
            FirmwareState::Idle => FirmwareStatus::Idle,
            FirmwareState::New(_) => FirmwareStatus::Idle,
            FirmwareState::Downloading(_) => FirmwareStatus::Downloading,
            FirmwareState::DownloadSleep(_) => FirmwareStatus::Downloading,
            FirmwareState::WaitingForTransactionToFinish(_) => FirmwareStatus::Downloaded,
            FirmwareState::Installing => FirmwareStatus::Installing,
        };
        self.send_firmware_status_notification(status);
    }
}
