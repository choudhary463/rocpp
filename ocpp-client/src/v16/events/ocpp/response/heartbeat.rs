use rocpp_core::v16::messages::heart_beat::HeartbeatResponse;

use crate::v16::{cp::{ChargePoint, OcppError}, interfaces::ChargePointInterface, state_machine::heartbeat::HeartbeatState};


impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn heartbeat_response(&mut self, res: Result<HeartbeatResponse, OcppError>) {
        match res {
            Ok(t) => {
                self.set_time(t.current_time).await;
                if let HeartbeatState::WaitingForResponse = &self.heartbeat_state {
                    self.set_sleep_heartbeat().await;
                }
            }
            Err(e) => {
                log::error!("heartbeat_response error: {:?}", e);
            }
        }
    }
}
