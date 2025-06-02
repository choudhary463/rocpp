use ocpp_core::v16::messages::status_notification::StatusNotificationResponse;

use crate::v16::{
    drivers::{database::Database, hardware_interface::HardwareInterface},
    cp::core::{ChargePointCore, OcppError},
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
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
