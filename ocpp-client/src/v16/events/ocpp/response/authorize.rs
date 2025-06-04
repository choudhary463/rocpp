use rocpp_core::v16::messages::authorize::AuthorizeResponse;

use crate::v16::{cp::{ChargePoint, OcppError}, interfaces::ChargePointInterface};


impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn authorized_response(&mut self, res: Result<AuthorizeResponse, OcppError>) {
        if let Some((connector_id, id_tag)) = self.pending_auth_requests.pop_front() {
            match res {
                Ok(t) => {
                    if t.id_tag_info.is_valid(self.get_time().await) {
                        self.handle_id_tag_authorized(
                            connector_id,
                            id_tag.clone(),
                            t.id_tag_info.parent_id_tag.clone(),
                        ).await;
                    }
                    self.update_cache(id_tag, t.id_tag_info).await;
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
