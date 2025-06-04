use rocpp_core::v16::messages::firmware_status_notification::FirmwareStatusNotificationResponse;

use crate::v16::{
    cp::{ChargePoint, OcppError},
    interfaces::ChargePointInterface,
};

impl<I: ChargePointInterface> ChargePoint<I> {
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
