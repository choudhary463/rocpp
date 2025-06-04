use alloc::string::String;
use rocpp_core::v16::{messages::authorize::AuthorizeRequest, types::IdTagInfo};

use crate::v16::{cp::ChargePoint, interfaces::ChargePointInterface};

use super::{call::CallAction, connector::ConnectorState};

#[derive(Clone)]
pub(crate) enum LocalListChange {
    Upsert { id_tag: String, info: IdTagInfo },
    Delete { id_tag: String },
}

impl LocalListChange {
    pub fn get_id_tag(&self) -> &str {
        match self {
            Self::Upsert { id_tag, .. } => &id_tag,
            Self::Delete { id_tag } => &id_tag,
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

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn send_authorize_request(&mut self, connector_id: usize, id_tag: String) {
        self.pending_auth_requests
            .push_back((connector_id, id_tag.clone()));
        let payload = AuthorizeRequest { id_tag };
        self.enqueue_call(CallAction::Authorize, payload).await;
    }

    pub(crate) async fn update_cache(&mut self, id_tag: String, info: IdTagInfo) {
        if !self.configs.authorization_cache_enabled.value {
            return;
        }
        if self
            .interface
            .db_get_from_local_list(&id_tag)
            .await
            .is_some()
        {
            // skip
        } else {
            self.interface.db_update_cache(&id_tag, info).await;
            return;
        }
    }

    pub(crate) async fn evaluate_id_tag_auth(
        &mut self,
        id_tag: String,
        connector_id: usize,
    ) -> AuthorizeStatus {
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
        let mut info = self
            .interface
            .db_get_from_local_list(&id_tag)
            .await
            .filter(|_| self.configs.local_auth_list_enabled.value);
        if info.is_none() {
            info = self
                .interface
                .db_get_from_cache(&id_tag)
                .await
                .filter(|_| self.configs.authorization_cache_enabled.value)
        }
        let now = self.get_time().await;
        let mut parent_id_tag = info.and_then(|t| t.is_valid(now).then(|| t.parent_id_tag.clone()));

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
}
