use chrono::Utc;
use ocpp_core::v16::messages::{
    boot_notification::BootNotificationResponse, heart_beat::HeartbeatResponse,
};

pub trait WithNowTimestamp {
    fn with_now(self) -> Self;
}

impl WithNowTimestamp for HeartbeatResponse {
    fn with_now(self) -> Self {
        Self {
            current_time: Utc::now(),
        }
    }
}

impl WithNowTimestamp for BootNotificationResponse {
    fn with_now(self) -> Self {
        Self {
            current_time: Utc::now(),
            ..self
        }
    }
}
