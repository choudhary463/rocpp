use ocpp_core::v16::messages::authorize::AuthorizeResponse;

use crate::v16::{
    drivers::{database::Database, hardware_interface::HardwareInterface},
    cp::core::{ChargePointCore, OcppError},
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn authorized_response(&mut self, res: Result<AuthorizeResponse, OcppError>) {
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
