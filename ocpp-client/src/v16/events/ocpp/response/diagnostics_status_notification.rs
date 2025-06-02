use ocpp_core::v16::messages::diagnostics_status_notification::DiagnosticsStatusNotificationResponse;

use crate::v16::{
    drivers::{database::Database, hardware_interface::HardwareInterface},
    cp::core::{ChargePointCore, OcppError},
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn diagnostics_status_notification_response(
        &mut self,
        res: Result<DiagnosticsStatusNotificationResponse, OcppError>,
    ) {
        match res {
            Ok(_) => {}
            Err(e) => {
                log::error!("diagnostics_status_notification_response error: {:?}", e);
            }
        }
    }
}
