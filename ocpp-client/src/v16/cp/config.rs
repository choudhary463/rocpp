use alloc::{string::String, vec::Vec};
use ocpp_core::v16::messages::boot_notification::BootNotificationRequest;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ChargePointConfig {
    pub cms_url: String,
    pub call_timeout: u64,
    pub max_cache_len: usize,
    pub boot_info: BootNotificationRequest,
    pub default_ocpp_configs: Vec<(String, String)>,
    pub clear_db: bool,
    pub seed: u64,
}