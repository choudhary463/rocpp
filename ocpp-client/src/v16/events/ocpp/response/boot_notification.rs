use rocpp_core::v16::{
    messages::boot_notification::BootNotificationResponse, types::RegistrationStatus,
};

use crate::v16::{
    cp::{ChargePoint, OcppError},
    interfaces::{ChargePointInterface, TimerId},
    state_machine::boot::BootState,
};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn boot_notification_response(
        &mut self,
        res: Result<BootNotificationResponse, OcppError>,
    ) {
        match &self.boot_state {
            BootState::WaitingForResponse => {
                let prev = self.registration_status.clone();
                let backoff;
                match res {
                    Ok(t) => {
                        self.set_time(t.current_time).await;
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
                            .update(backoff, &mut self.interface)
                            .await;
                    }
                    if prev != RegistrationStatus::Accepted {
                        self.notify_online().await;
                    }
                    self.boot_state = BootState::Idle;
                } else {
                    if prev == RegistrationStatus::Accepted {
                        self.notify_offline().await;
                    }
                    let timeout = if backoff == 0 { 2 } else { backoff };
                    self.add_timeout(TimerId::Boot, timeout).await;
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
