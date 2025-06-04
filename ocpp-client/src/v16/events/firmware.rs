use rocpp_core::v16::types::{FirmwareStatus, Reason, ResetType};

use crate::v16::{cp::ChargePoint, interfaces::{ChargePointInterface, TimerId}, state_machine::firmware::{FirmwareInstallStatus, FirmwareState}};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn firmware_download_response(&mut self, res: bool) {
        match core::mem::replace(&mut self.firmware_state, FirmwareState::Idle) {
            FirmwareState::Downloading(mut t) => match res {
                true => {
                    self.send_firmware_status_notification(FirmwareStatus::Downloaded).await;
                    if self.active_local_transactions.iter().all(|f| f.is_none()) {
                        self.try_firmware_install().await;
                    } else {
                        self.firmware_state =
                            FirmwareState::WaitingForTransactionToFinish;
                        for connector_id in 0..self.configs.number_of_connectors.value {
                            self.sync_connector_states(connector_id, None, None).await;
                        }
                    }
                }
                false => {
                    t.retry_left -= 1;
                    if t.retry_left > 0 && t.retry_interval > 0 {
                        self.add_timeout(TimerId::Firmware, t.retry_interval).await;
                        self.firmware_state = FirmwareState::DownloadSleep(t);
                    } else {
                        self.try_firmware_download(t).await;
                    }
                }
            },
            _ => {
                unreachable!();
            }
        }
    }
    pub async fn firmware_install_response(&mut self, res: bool) {
        let state = match res {
            true => FirmwareInstallStatus::InstallationSuccess,
            false => FirmwareInstallStatus::InstallationFailed,
        };
        self.firmware_state = FirmwareState::Idle;
        self.interface.db_change_firmware_state(state).await;
        self.reset(ResetType::Soft, Some(Reason::Reboot)).await;
    }
}
