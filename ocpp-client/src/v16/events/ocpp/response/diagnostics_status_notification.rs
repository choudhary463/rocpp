use ocpp_core::v16::messages::diagnostics_status_notification::DiagnosticsStatusNotificationResponse;

use crate::v16::{
    interface::{Database, Secc},
    cp::{ChargePointCore, OcppError},
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
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
