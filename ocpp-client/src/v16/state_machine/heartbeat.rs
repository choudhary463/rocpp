use ocpp_core::v16::messages::heart_beat::HeartbeatRequest;

use crate::v16::{
    cp::core::ChargePointCore, drivers::{database::Database, hardware_interface::HardwareInterface, timers::TimerId}
};

use super::call::CallAction;

pub(crate) enum HeartbeatState {
    Idle,
    Sleeping,
    WaitingForResponse,
}

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn set_sleep_heartbeat(&mut self) {
        let interval = if self.configs.heartbeat_interval.value == 0 {
            2
        } else {
            self.configs.heartbeat_interval.value
        };
        self.add_timeout(TimerId::Heartbeat, interval);
        self.heartbeat_state = HeartbeatState::Sleeping;
    }
    pub(crate) fn send_heartbeat(&mut self) {
        self.enqueue_call(CallAction::Heartbeat, HeartbeatRequest {});
        self.heartbeat_state = HeartbeatState::WaitingForResponse;
    }
    pub(crate) fn heartbeat_activity(&mut self) {
        match &self.heartbeat_state {
            HeartbeatState::Sleeping => {
                self.set_sleep_heartbeat();
            }
            HeartbeatState::WaitingForResponse => {
                // already sent, will reset timer once response is received
            }
            _ => {
                // can reach here before registation accepted
            }
        }
    }
    pub(crate) fn on_heartbeat_online(&mut self) {
        match &self.heartbeat_state {
            HeartbeatState::Idle => {
                self.set_sleep_heartbeat();
            }
            _ => {
                // heartbet state must be idle in offline state
                unreachable!();
            }
        }
    }
    pub(crate) fn on_heartbeat_offline(&mut self) {
        match &self.heartbeat_state {
            HeartbeatState::Sleeping => {
                self.remove_timeout(TimerId::Heartbeat);
            }
            HeartbeatState::Idle => {}
            _ => {
                // heartbeat_response will be called before this
                unreachable!();
            }
        }
        self.heartbeat_state = HeartbeatState::Idle;
    }
    pub(crate) fn trigger_heartbeat(&mut self) {
        match &self.heartbeat_state {
            HeartbeatState::Idle => {
                self.enqueue_call(CallAction::Heartbeat, HeartbeatRequest {});
            }
            HeartbeatState::Sleeping => {
                self.remove_timeout(TimerId::Heartbeat);
                self.send_heartbeat();
            }
            HeartbeatState::WaitingForResponse => {
                // already sent
            }
        }
    }
}
