use ocpp_core::v16::messages::firmware_status_notification::FirmwareStatusNotificationResponse;

use crate::v16::{
    interface::{Database, Secc},
    cp::{ChargePointCore, OcppError},
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub(crate) fn firmware_status_notification_response(
        &mut self,
        res: Result<FirmwareStatusNotificationResponse, OcppError>,
    ) {
        match res {
            Ok(_) => {}
            Err(e) => {
                log::error!("firmware_status_notification_response error: {:?}", e);
            }
        }
    }
}
