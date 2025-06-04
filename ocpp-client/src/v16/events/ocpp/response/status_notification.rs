use rocpp_core::v16::messages::status_notification::StatusNotificationResponse;

use crate::v16::{cp::{ChargePoint, OcppError}, interfaces::ChargePointInterface};


impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) fn status_notification_response(
        &mut self,
        res: Result<StatusNotificationResponse, OcppError>,
    ) {
        match res {
            Ok(_) => {}
            Err(e) => {
                log::error!("status_notification_response error: {:?}", e);
            }
        }
    }
}
