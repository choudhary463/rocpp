use ocpp_core::v16::messages::heart_beat::HeartbeatResponse;

use crate::v16::{
    interface::{Database, Secc},
    state_machine::{
        core::{ChargePointCore, OcppError},
        heartbeat::HeartbeatState,
    },
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn heartbeat_response(&mut self, res: Result<HeartbeatResponse, OcppError>) {
        match res {
            Ok(t) => {
                self.set_time(t.current_time);
                if let HeartbeatState::WaitingForResponse = &self.heartbeat_state {
                    self.set_sleep_heartbeat();
                }
            }
            Err(e) => {
                log::error!("heartbeat_response error: {:?}", e);
            }
        }
    }
}
