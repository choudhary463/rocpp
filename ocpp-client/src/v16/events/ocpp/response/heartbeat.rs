use ocpp_core::v16::messages::heart_beat::HeartbeatResponse;

use crate::v16::{
    drivers::{database::Database, hardware_interface::HardwareInterface},
    state_machine::{
        heartbeat::HeartbeatState,
    },
    cp::core::{ChargePointCore, OcppError},
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn heartbeat_response(&mut self, res: Result<HeartbeatResponse, OcppError>) {
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
