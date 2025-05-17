use ocpp_core::v16::types::{FirmwareStatus, Reason, ResetType};

use crate::v16::{
    interface::{Database, Secc},
    services::timeout::TimerId,
    state_machine::{
        core::ChargePointCore,
        firmware::{FirmwareInstallStatus, FirmwareState},
    },
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn firmware_download_response(&mut self, res: Option<Vec<u8>>) {
        match std::mem::replace(&mut self.firmware_state, FirmwareState::Idle) {
            FirmwareState::Downloading(mut t) => match res {
                Some(firmware_image) => {
                    self.send_firmware_status_notification(FirmwareStatus::Downloaded);
                    if self.active_local_transactions.iter().all(|f| f.is_none()) {
                        self.try_firmware_install(firmware_image);
                    } else {
                        self.firmware_state =
                            FirmwareState::WaitingForTransactionToFinish(firmware_image);
                        for connector_id in 0..self.configs.number_of_connectors.value {
                            self.sync_connector_states(connector_id, None, None);
                        }
                    }
                }
                None => {
                    t.retry_left -= 1;
                    if t.retry_left > 0 && t.retry_interval > 0 {
                        self.add_timeout(TimerId::Firmware, t.retry_interval);
                        self.firmware_state = FirmwareState::DownloadSleep(t);
                    } else {
                        self.try_firmware_download(t);
                    }
                }
            },
            _ => {
                unreachable!();
            }
        }
    }
    pub fn firmware_install_response(&mut self, res: bool) {
        let state = match res {
            true => FirmwareInstallStatus::InstallationSuccess,
            false => FirmwareInstallStatus::InstallationFailed,
        };
        self.db.db_change_firmware_state(state);
        self.reset(ResetType::Soft, Some(Reason::Reboot));
    }
}
