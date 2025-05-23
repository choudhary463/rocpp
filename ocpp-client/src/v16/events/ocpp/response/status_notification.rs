use ocpp_core::v16::messages::status_notification::StatusNotificationResponse;

use crate::v16::{
    interface::{Database, Secc},
    cp::{ChargePointCore, OcppError},
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
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
