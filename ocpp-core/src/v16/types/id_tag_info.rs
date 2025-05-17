use chrono::{DateTime, Utc};

use super::AuthorizationStatus;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IdTagInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id_tag: Option<String>,
    pub status: AuthorizationStatus,
}

impl IdTagInfo {
    pub fn is_valid(&self, current_time: Option<DateTime<Utc>>) -> bool {
        if self.status == AuthorizationStatus::Accepted {
            if let Some(expiry_date) = self.expiry_date {
                if let Some(time_now) = current_time {
                    time_now <= expiry_date
                } else {
                    true
                }
            } else {
                true
            }
        } else {
            false
        }
    }
}
