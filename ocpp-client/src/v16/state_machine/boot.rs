use ocpp_core::v16::types::RegistrationStatus;

use crate::v16::{
    interface::{Database, Secc, TimerId},
    cp::ChargePointCore,
};

use super::call::CallAction;

pub(crate) enum BootState {
    Idle,
    Sleeping,
    WaitingForResponse,
}

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub(crate) fn send_boot_notification(&mut self) {
        self.enqueue_call(CallAction::BootNotification, self.boot_info.clone());
        self.boot_state = BootState::WaitingForResponse
    }

    pub(crate) fn notify_online(&mut self) {
        self.on_heartbeat_online();
        self.on_transaction_online();
        self.on_status_notification_online();
        self.on_firmware_online();
    }

    pub(crate) fn notify_offline(&mut self) {
        self.on_heartbeat_offline();
        self.on_status_notification_offline();
    }

    pub(crate) fn on_boot_connected(&mut self) {
        match &self.boot_state {
            BootState::Idle => {
                if self.registration_status != RegistrationStatus::Accepted {
                    self.send_boot_notification();
                } else {
                    self.notify_online();
                }
            }
            _ => {
                unreachable!();
            }
        }
    }

    pub(crate) fn on_boot_disconnected(&mut self) {
        match &self.boot_state {
            BootState::Sleeping => {
                self.remove_timeout(TimerId::Boot);
            }
            BootState::WaitingForResponse => {
                // got_boot_response will be called before this
                unreachable!();
            }
            _ => {}
        }
        if self.registration_status == RegistrationStatus::Accepted {
            self.notify_offline();
        }
        self.boot_state = BootState::Idle;
    }

    pub(crate) fn trigger_boot(&mut self) {
        match &self.boot_state {
            BootState::Idle => {
                self.send_boot_notification();
            }
            BootState::Sleeping => {
                self.remove_timeout(TimerId::Boot);
                self.send_boot_notification();
            }
            BootState::WaitingForResponse => {
                // already sent
            }
        }
    }
}
