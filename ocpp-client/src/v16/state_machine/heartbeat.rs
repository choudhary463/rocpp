use rocpp_core::v16::messages::heart_beat::HeartbeatRequest;

use crate::v16::{
    cp::ChargePoint,
    interfaces::{ChargePointInterface, TimerId},
};

use super::call::CallAction;

pub(crate) enum HeartbeatState {
    Idle,
    Sleeping,
    WaitingForResponse,
}

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn set_sleep_heartbeat(&mut self) {
        let interval = if self.configs.heartbeat_interval.value == 0 {
            2
        } else {
            self.configs.heartbeat_interval.value
        };
        self.add_timeout(TimerId::Heartbeat, interval).await;
        self.heartbeat_state = HeartbeatState::Sleeping;
    }
    pub(crate) async fn send_heartbeat(&mut self) {
        self.enqueue_call(CallAction::Heartbeat, HeartbeatRequest {})
            .await;
        self.heartbeat_state = HeartbeatState::WaitingForResponse;
    }
    pub(crate) async fn heartbeat_activity(&mut self) {
        match &self.heartbeat_state {
            HeartbeatState::Sleeping => {
                self.set_sleep_heartbeat().await;
            }
            HeartbeatState::WaitingForResponse => {
                // already sent, will reset timer once response is received
            }
            _ => {
                // can reach here before registation accepted
            }
        }
    }
    pub(crate) async fn on_heartbeat_online(&mut self) {
        match &self.heartbeat_state {
            HeartbeatState::Idle => {
                self.set_sleep_heartbeat().await;
            }
            _ => {
                // heartbet state must be idle in offline state
                unreachable!();
            }
        }
    }
    pub(crate) async fn on_heartbeat_offline(&mut self) {
        match &self.heartbeat_state {
            HeartbeatState::Sleeping => {
                self.remove_timeout(TimerId::Heartbeat).await;
            }
            HeartbeatState::Idle => {}
            _ => {
                // heartbeat_response will be called before this
                unreachable!();
            }
        }
        self.heartbeat_state = HeartbeatState::Idle;
    }
    pub(crate) async fn trigger_heartbeat(&mut self) {
        match &self.heartbeat_state {
            HeartbeatState::Idle => {
                self.enqueue_call(CallAction::Heartbeat, HeartbeatRequest {})
                    .await;
            }
            HeartbeatState::Sleeping => {
                self.remove_timeout(TimerId::Heartbeat).await;
                self.send_heartbeat().await;
            }
            HeartbeatState::WaitingForResponse => {
                // already sent
            }
        }
    }
}
