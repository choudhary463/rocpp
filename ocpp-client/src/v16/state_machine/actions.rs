use chrono::{DateTime, Utc};
use ocpp_core::v16::types::ResetType;
use std::time::{Duration, Instant};

use crate::v16::{
    interface::{Database, Secc},
    services::timeout::TimerId,
    state_machine::core::ChargePointCore,
};

#[derive(Debug)]
pub(crate) enum CoreActions {
    Connect(String),
    SendWsMsg(String),
    StartDiagnosticUpload {
        location: String,
        file_name: String,
        start_time: Option<DateTime<Utc>>,
        stop_time: Option<DateTime<Utc>>,
    },
    CancelDiagnosticUpload,
    DownloadFirmware(String),
    InstallFirmware(Vec<u8>),
    AddTimeout(TimerId, Instant),
    RemoveTimeout(TimerId),
    SoftReset,
}

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn connect(&mut self, cms_url: String) {
        self.queued_actions.push_back(CoreActions::Connect(cms_url));
    }

    pub fn send_ws_msg(&mut self, msg: String) {
        if self.ws_connected && self.pending_reset != Some(ResetType::Hard) {
            self.queued_actions.push_back(CoreActions::SendWsMsg(msg));
            self.heartbeat_activity();
        }
    }

    pub fn start_diagnostics_upload(
        &mut self,
        location: String,
        file_name: String,
        start_time: Option<DateTime<Utc>>,
        stop_time: Option<DateTime<Utc>>,
    ) {
        self.queued_actions
            .push_back(CoreActions::StartDiagnosticUpload {
                location,
                file_name,
                start_time,
                stop_time,
            });
    }

    pub fn cancel_diagnostics_upload(&mut self) {
        self.queued_actions
            .push_back(CoreActions::CancelDiagnosticUpload);
    }

    pub fn download_firmware(&mut self, firmware_url: String) {
        self.queued_actions
            .push_back(CoreActions::DownloadFirmware(firmware_url));
    }

    pub fn install_firmware(&mut self, firmware_image: Vec<u8>) {
        self.queued_actions
            .push_back(CoreActions::InstallFirmware(firmware_image));
    }

    pub fn add_timeout(&mut self, timer_id: TimerId, timeout_secs: u64) {
        self.queued_actions.push_back(CoreActions::AddTimeout(
            timer_id,
            Instant::now() + Duration::from_secs(timeout_secs),
        ));
    }

    pub fn remove_timeout(&mut self, timer_id: TimerId) {
        self.queued_actions
            .push_back(CoreActions::RemoveTimeout(timer_id));
    }

    pub fn soft_reset(&mut self) {
        self.queued_actions.push_back(CoreActions::SoftReset);
    }
}
