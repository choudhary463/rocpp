use alloc::string::String;
use ocpp_core::{
    format::{
        frame::{Call, CallError},
        message::{EncodeDecode, OcppMessage},
    },
    v16::{protocol_error::ProtocolError, types::RegistrationStatus},
};

use crate::v16::{
    cp::core::ChargePointCore, drivers::{database::Database, hardware_interface::HardwareInterface}
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub fn ws_connected_helper(&mut self) {
        self.ws_connected = true;
        self.on_boot_connected();
    }

    pub fn ws_disconnected_helper(&mut self) {
        self.ws_connected = false;
        self.on_outgoing_offline();
        self.on_boot_disconnected();
        self.connect(self.cms_url.clone());
    }

    pub(crate) fn send_error(&mut self, uid: String, err: ProtocolError) {
        let err = CallError::new(uid, err);
        self.send_ws_msg(err.encode());
    }

    pub fn got_ws_msg_helper(&mut self, msg: String) {
        self.heartbeat_activity();

        let res = OcppMessage::<ProtocolError>::decode(msg);
        match res {
            OcppMessage::Call(call) => match call.action.as_str() {
                "CancelReservation" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.cancel_reservation_ocpp(unique_id, req);
                    });
                }
                "ChangeAvailability" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.change_availability_ocpp(unique_id, req);
                    });
                }
                "ChangeConfiguration" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.change_configuration_ocpp(unique_id, req);
                    });
                }
                "ClearCache" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.clear_cache_ocpp(unique_id, req);
                    });
                }
                "ClearChargingProfile" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.clear_charging_profile_ocpp(unique_id, req);
                    });
                }
                "DataTransfer" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.data_transfer_ocpp(unique_id, req);
                    });
                }
                "GetCompositeSchedule" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.get_composite_schedule_ocpp(unique_id, req);
                    });
                }
                "GetConfiguration" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.get_configuration_ocpp(unique_id, req);
                    });
                }
                "GetDiagnostics" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.get_diagnostics_ocpp(unique_id, req);
                    });
                }
                "GetLocalListVersion" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.get_local_list_version_ocpp(unique_id, req);
                    });
                }
                "RemoteStartTransaction" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.remote_start_transaction_ocpp(unique_id, req);
                    });
                }
                "RemoteStopTransaction" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.remote_stop_transaction_ocpp(unique_id, req);
                    });
                }
                "ReserveNow" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.reserve_now_ocpp(unique_id, req);
                    });
                }
                "Reset" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.reset_ocpp(unique_id, req);
                    });
                }
                "SendLocalList" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.send_local_list_ocpp(unique_id, req);
                    });
                }
                "SetChargingProfile" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.set_charging_profile_ocpp(unique_id, req);
                    });
                }
                "TriggerMessage" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.trigger_message_ocpp(unique_id, req);
                    });
                }
                "UnlockConnector" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.unlock_connector_ocpp(unique_id, req);
                    });
                }
                "UpdateFirmware" => {
                    self.handle_call(call, |srv, unique_id, req| {
                        srv.update_firmware_ocpp(unique_id, req);
                    });
                }
                _ => {
                    self.send_error(call.unique_id, ProtocolError::NotSupported);
                }
            },
            OcppMessage::CallResponse(res) => {
                self.handle_call_response(Ok(res), true);
            }
            OcppMessage::Invalid(invalid) => {
                if let Some(uid) = invalid.unique_id {
                    self.send_error(uid, ProtocolError::FormationViolation);
                }
            }
        }
    }
    fn is_registered(&self, action: &str) -> bool {
        match self.registration_status {
            RegistrationStatus::Accepted => true,
            RegistrationStatus::Pending => {
                action != "RemoteStartTransaction" && action != "RemoteStopTransaction"
            }
            RegistrationStatus::Rejected => false,
        }
    }
    fn handle_call<T: for<'de> serde::Deserialize<'de>>(
        &mut self,
        call: Call,
        handler: impl FnOnce(&mut Self, String, T),
    ) {
        let uid = call.unique_id.clone();

        match serde_json::from_value::<T>(call.payload) {
            Ok(val) => {
                if self.is_registered(&call.action) {
                    handler(self, uid, val);
                } else {
                    self.send_error(uid, ProtocolError::SecurityError);
                }
            }
            Err(_) => {
                self.send_error(uid, ProtocolError::FormationViolation);
            }
        }
    }
}