use alloc::string::String;
use ocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::remote_start_transaction::{
            RemoteStartTransactionRequest, RemoteStartTransactionResponse,
        },
        types::RemoteStartStopStatus,
    },
};

use crate::v16::{
    drivers::{database::Database, hardware_interface::HardwareInterface},
    state_machine::{auth::AuthorizeStatus, connector::ConnectorState},
    cp::core::ChargePointCore
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    fn get_remote_start_info(
        &mut self,
        connector_id: Option<usize>,
        id_tag: &String,
    ) -> Option<usize> {
        if connector_id.map(|t| t == 0).unwrap_or(false) {
            return None;
        }
        for connector_id in match connector_id {
            Some(connector_id) => (connector_id - 1)..(connector_id),
            None => 0..self.configs.number_of_connectors.value,
        } {
            if let ConnectorState::Reserved { id_tag: tag, .. } = &self.connector_state[connector_id] {
                if tag == id_tag {
                    return Some(connector_id);
                }
            }
        }
        for connector_id in match connector_id {
            Some(connector_id) => (connector_id - 1)..(connector_id),
            None => 0..self.configs.number_of_connectors.value,
        } {
            if let ConnectorState::Plugged = &self.connector_state[connector_id] {
                return Some(connector_id);
            }
        }
        for connector_id in match connector_id {
            Some(connector_id) => (connector_id - 1)..(connector_id),
            None => 0..self.configs.number_of_connectors.value,
        } {
            if let ConnectorState::Idle = &self.connector_state[connector_id] {
                return Some(connector_id);
            }
        }
        None
    }
    pub(crate) fn remote_start_transaction_ocpp(
        &mut self,
        unique_id: String,
        req: RemoteStartTransactionRequest,
    ) {
        let mut auth_status = AuthorizeStatus::NotAuthorized;
        if let Some(connector_id) = self.get_remote_start_info(req.connector_id, &req.id_tag) {
            if self.configs.authorize_remote_transaction_requests.value {
                auth_status = self.evaluate_id_tag_auth(req.id_tag, connector_id);
            } else {
                auth_status = AuthorizeStatus::Authorized {
                    connector_id,
                    id_tag: req.id_tag,
                    parent_id_tag: None,
                };
            }
        }
        let status = match &auth_status {
            AuthorizeStatus::NotAuthorized => RemoteStartStopStatus::Rejected,
            _ => RemoteStartStopStatus::Accepted,
        };
        let payload = RemoteStartTransactionResponse { status };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode());

        match auth_status {
            AuthorizeStatus::NotAuthorized => {}
            AuthorizeStatus::Authorized {
                connector_id,
                id_tag,
                parent_id_tag,
            } => {
                self.handle_id_tag_authorized(connector_id, id_tag, parent_id_tag);
            }
            AuthorizeStatus::SendAuthorize {
                connector_id,
                id_tag,
            } => {
                self.send_authorize_request(connector_id, id_tag);
            }
        };
    }
}
