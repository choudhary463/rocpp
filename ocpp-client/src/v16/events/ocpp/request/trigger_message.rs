use alloc::string::String;
use ocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::trigger_message::{TriggerMessageRequest, TriggerMessageResponse},
        types::{MessageTrigger, RegistrationStatus, TriggerMessageStatus},
    },
};

use crate::v16::{
    drivers::{database::Database, hardware_interface::HardwareInterface},
    cp::core::ChargePointCore,
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn trigger_message_ocpp(&mut self, unique_id: String, req: TriggerMessageRequest) {
        let valid_connector_id = req
            .connector_id
            .map(|f| f <= self.configs.number_of_connectors.value)
            .unwrap_or_else(|| {
                !(matches!(
                    req.requested_message,
                    MessageTrigger::MeterValues | MessageTrigger::StatusNotification
                ))
            });
        let valid_message = !(self.registration_status == RegistrationStatus::Pending
            && req.requested_message == MessageTrigger::MeterValues);

        let status = if valid_connector_id && valid_message {
            TriggerMessageStatus::Accepted
        } else {
            TriggerMessageStatus::Rejected
        };
        let payload = TriggerMessageResponse { status };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode());

        if valid_connector_id && valid_message {
            match req.requested_message {
                MessageTrigger::BootNotification => {
                    self.trigger_boot();
                }
                MessageTrigger::DiagnosticsStatusNotification => {
                    self.trigger_diagnostics_status_notification();
                }
                MessageTrigger::FirmwareStatusNotification => {
                    self.trigger_firmware_status_notification();
                }
                MessageTrigger::Heartbeat => {
                    self.trigger_heartbeat();
                }
                MessageTrigger::MeterValues => {
                    let connector_id = req.connector_id.unwrap();
                    self.trigger_meter_values(connector_id);
                }
                MessageTrigger::StatusNotification => {
                    let connector_id = req.connector_id.unwrap();
                    self.trigger_status_notification(connector_id);
                }
            }
        }
    }
}
