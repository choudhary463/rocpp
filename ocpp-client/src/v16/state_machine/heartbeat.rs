use ocpp_core::v16::messages::heart_beat::HeartbeatRequest;

use crate::v16::{
    interface::{Database, Secc},
    services::timeout::TimerId,
};

use super::{call::CallAction, core::ChargePointCore};

pub(crate) enum HeartbeatState {
    Idle,
    Sleeping,
    WaitingForResponse,
}

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn set_sleep_heartbeat(&mut self) {
        let interval = if self.configs.heartbeat_interval.value == 0 {
            2
        } else {
            self.configs.heartbeat_interval.value
        };
        self.add_timeout(TimerId::Heartbeat, interval);
        self.heartbeat_state = HeartbeatState::Sleeping;
    }
    pub fn send_heartbeat(&mut self) {
        self.enqueue_call(CallAction::Heartbeat, HeartbeatRequest {});
        self.heartbeat_state = HeartbeatState::WaitingForResponse;
    }
    pub fn heartbeat_activity(&mut self) {
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
    pub fn on_heartbeat_online(&mut self) {
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
    pub fn on_heartbeat_offline(&mut self) {
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
    pub fn trigger_heartbeat(&mut self) {
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
