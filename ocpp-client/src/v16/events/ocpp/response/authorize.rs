use ocpp_core::v16::messages::authorize::AuthorizeResponse;

use crate::v16::{
    interface::{Database, Secc},
    state_machine::core::{ChargePointCore, OcppError},
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn authorized_response(&mut self, res: Result<AuthorizeResponse, OcppError>) {
        if let Some((connector_id, id_tag)) = self.pending_auth_requests.pop_front() {
            match res {
                Ok(t) => {
                    if t.id_tag_info.is_valid(self.get_time()) {
                        self.handle_id_tag_authorized(
                            connector_id,
                            id_tag.clone(),
                            t.id_tag_info.parent_id_tag.clone(),
                        );
                    }
                    self.update_cache(id_tag, t.id_tag_info);
                }
                Err(e) => {
                    log::error!("authorized_response error: {:?}", e);
                }
            }
        } else {
            unreachable!();
        }
    }
}
