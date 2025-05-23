use alloc::{string::{String, ToString}, vec};
use chrono::{DateTime, Utc};
use ocpp_core::v16::{
    messages::authorize::AuthorizeRequest,
    types::{AuthorizationStatus, IdTagInfo},
};

use crate::v16::{interface::{Database, Secc}, cp::ChargePointCore};

use super::{call::CallAction, connector::ConnectorState};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct CachedEntry {
    pub info: IdTagInfo,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub(crate) enum LocalListChange {
    Upsert { id_tag: String, info: IdTagInfo },
    Delete { id_tag: String },
}

impl LocalListChange {
    pub fn get_id_tag(&self) -> &str {
        match self {
            Self::Upsert { id_tag, .. } => &id_tag,
            Self::Delete { id_tag } => &id_tag
        }
    }
}

pub(crate) enum AuthorizeStatus {
    NotAuthorized,
    Authorized {
        connector_id: usize,
        id_tag: String,
        parent_id_tag: Option<String>,
    },
    SendAuthorize {
        connector_id: usize,
        id_tag: String,
    },
}

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub(crate) fn send_authorize_request(&mut self, connector_id: usize, id_tag: String) {
        self.pending_auth_requests
            .push_back((connector_id, id_tag.clone()));
        let payload = AuthorizeRequest { id_tag };
        self.enqueue_call(CallAction::Authorize, payload);
    }

    pub(crate) fn remove_from_cache(&mut self, id_tag: &str) {
        self.db.db_delete_cache(vec![id_tag.into()]);
        if self.authorization_cache.remove(id_tag).is_some() {
            if let Some(pos) = self.cache_usage_order.iter().position(|t| t == id_tag) {
                self.cache_usage_order.remove(pos);
            }
        }
    }

    pub(crate) fn update_cache(&mut self, id_tag: String, info: IdTagInfo) {
        if !self.configs.authorization_cache_enabled.value {
            return;
        }
        let now = self.get_time().unwrap_or(self.default_time());

        if let Some(entry) = self.authorization_cache.get_mut(&id_tag) {
            entry.info = info.clone();
            entry.updated_at = now;
            self.mark_recently_used(&id_tag);
        } else if self.local_list_entries.contains_key(&id_tag) {
            // skip
        } else {
            while self.authorization_cache.len() >= self.max_cache_len {
                self.evict_one();
            }

            let entry = CachedEntry {
                info: info.clone(),
                updated_at: now,
            };

            self.authorization_cache
                .insert(id_tag.clone(), entry.clone());
            self.cache_usage_order.push_front(id_tag.clone());

            self.db.db_update_cache(id_tag, entry);
            return;
        }
    }

    pub(crate) fn evaluate_id_tag_auth(&mut self, id_tag: String, connector_id: usize) -> AuthorizeStatus {
        let is_auth_req_in_queue = self
            .pending_auth_requests
            .iter()
            .any(|f| f.0 == connector_id);
        if is_auth_req_in_queue {
            // ??? drop req
            return AuthorizeStatus::NotAuthorized;
        }
        let in_transaction = match &self.connector_state[connector_id] {
            ConnectorState::Transaction {
                id_tag: transaction_id_tag,
                ..
            } => {
                if *transaction_id_tag == id_tag {
                    return AuthorizeStatus::Authorized {
                        connector_id,
                        id_tag,
                        parent_id_tag: None,
                    };
                }
                true
            }
            ConnectorState::Finishing => return AuthorizeStatus::NotAuthorized,
            ConnectorState::Unavailable(_) => return AuthorizeStatus::NotAuthorized,
            ConnectorState::Faulty => return AuthorizeStatus::NotAuthorized,
            _ => false,
        };
        let info = self
            .local_list_entries
            .get(&id_tag)
            .filter(|_| self.configs.local_auth_list_enabled.value)
            .or_else(|| {
                self.authorization_cache
                    .get(&id_tag)
                    .map(|e| &e.info)
                    .filter(|_| self.configs.authorization_cache_enabled.value)
            });
        let mut parent_id_tag =
            info.and_then(|t| t.is_valid(self.get_time()).then(|| t.parent_id_tag.clone()));

        let is_online = self.call_permission();

        if !in_transaction {
            if (!is_online && !self.configs.local_authorize_offline.value)
                || (is_online && !self.configs.local_pre_authorize.value)
            {
                parent_id_tag = None;
            }
            if parent_id_tag.is_none()
                && !is_online
                && self.configs.allow_offline_transaction_for_unknown_id.value
            {
                parent_id_tag = Some(None);
            }
        }
        if let Some(parent_id_tag) = parent_id_tag {
            return AuthorizeStatus::Authorized {
                connector_id,
                id_tag,
                parent_id_tag,
            };
        }
        if is_online {
            return AuthorizeStatus::SendAuthorize {
                connector_id,
                id_tag,
            };
        }
        AuthorizeStatus::NotAuthorized
    }

    fn mark_recently_used(&mut self, id_tag: &str) {
        if let Some(pos) = self.cache_usage_order.iter().position(|t| t == id_tag) {
            self.cache_usage_order.remove(pos);
            self.cache_usage_order.push_back(id_tag.to_string());
        }
    }

    fn evict_one(&mut self) {
        let evict_id_tag = self
            .cache_usage_order
            .iter()
            .find(|id_tag| {
                self.authorization_cache
                    .get(*id_tag)
                    .map(|entry| !matches!(entry.info.status, AuthorizationStatus::Accepted))
                    .unwrap_or(false)
            })
            .cloned()
            .or_else(|| self.cache_usage_order.pop_front());

        if let Some(id_tag) = evict_id_tag {
            self.remove_from_cache(&id_tag);
        }
    }
}
