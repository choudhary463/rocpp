use rocpp_core::v16::types::RegistrationStatus;

use crate::v16::{cp::ChargePoint, interfaces::{ChargePointInterface, TimerId}};

use super::call::CallAction;

pub(crate) enum BootState {
    Idle,
    Sleeping,
    WaitingForResponse,
}

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn send_boot_notification(&mut self) {
        self.enqueue_call(CallAction::BootNotification, self.boot_info.clone()).await;
        self.boot_state = BootState::WaitingForResponse
    }

    pub(crate) async fn notify_online(&mut self) {
        self.on_heartbeat_online().await;
        self.on_transaction_online().await;
        self.on_status_notification_online().await;
        self.on_firmware_online().await;
    }

    pub(crate) async fn notify_offline(&mut self) {
        self.on_heartbeat_offline().await;
        self.on_status_notification_offline();
    }

    pub(crate) async fn on_boot_connected(&mut self) {
        match &self.boot_state {
            BootState::Idle => {
                if self.registration_status != RegistrationStatus::Accepted {
                    self.send_boot_notification().await;
                } else {
                    self.notify_online().await;
                }
            }
            _ => {
                unreachable!();
            }
        }
    }

    pub(crate) async fn on_boot_disconnected(&mut self) {
        match &self.boot_state {
            BootState::Sleeping => {
                self.remove_timeout(TimerId::Boot).await;
            }
            BootState::WaitingForResponse => {
                // got_boot_response will be called before this
                unreachable!();
            }
            _ => {}
        }
        if self.registration_status == RegistrationStatus::Accepted {
            self.notify_offline().await;
        }
        self.boot_state = BootState::Idle;
    }

    pub(crate) async fn trigger_boot(&mut self) {
        match &self.boot_state {
            BootState::Idle => {
                self.send_boot_notification().await;
            }
            BootState::Sleeping => {
                self.remove_timeout(TimerId::Boot).await;
                self.send_boot_notification().await;
            }
            BootState::WaitingForResponse => {
                // already sent
            }
        }
    }
}
