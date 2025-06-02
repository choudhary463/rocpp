use ocpp_core::v16::{
    messages::boot_notification::BootNotificationResponse, types::RegistrationStatus,
};

use crate::v16::{
    cp::core::{ChargePointCore, OcppError}, drivers::{database::Database, hardware_interface::HardwareInterface, timers::TimerId}, state_machine::boot::BootState
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn boot_notification_response(&mut self, res: Result<BootNotificationResponse, OcppError>) {
        match &self.boot_state {
            BootState::WaitingForResponse => {
                let prev = self.registration_status.clone();
                let backoff;
                match res {
                    Ok(t) => {
                        self.set_time(t.current_time);
                        self.registration_status = t.status;
                        backoff = t.interval;
                    }
                    Err(e) => {
                        log::error!("boot_notification_response error: {:?}", e);
                        backoff = 0;
                    }
                }
                if self.registration_status == RegistrationStatus::Accepted {
                    if backoff > 0 {
                        self.configs
                            .heartbeat_interval
                            .update(backoff, &mut self.db);
                    }
                    if prev != RegistrationStatus::Accepted {
                        self.notify_online();
                    }
                    self.boot_state = BootState::Idle;
                } else {
                    if prev == RegistrationStatus::Accepted {
                        self.notify_offline();
                    }
                    let timeout = if backoff == 0 { 2 } else { backoff };
                    self.add_timeout(TimerId::Boot, timeout);
                    self.boot_state = BootState::Sleeping;
                }
            }
            _ => {
                // boot response received while not in WaitingForResponse state
                unreachable!();
            }
        }
    }
}
