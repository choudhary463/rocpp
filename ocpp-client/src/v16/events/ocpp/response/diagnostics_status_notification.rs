use rocpp_core::v16::messages::diagnostics_status_notification::DiagnosticsStatusNotificationResponse;

use crate::v16::{cp::{ChargePoint, OcppError}, interfaces::ChargePointInterface};


impl<I: ChargePointInterface> ChargePoint<I> {
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
