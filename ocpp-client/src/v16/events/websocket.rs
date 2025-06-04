use alloc::string::String;
use rocpp_core::{
    format::{
        frame::{Call, CallError},
        message::{EncodeDecode, OcppMessage},
    },
    v16::{protocol_error::ProtocolError, types::RegistrationStatus},
};

use crate::v16::{cp::ChargePoint, interfaces::ChargePointInterface};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub async fn ws_connected(&mut self) {
        self.ws_connected = true;
        self.on_boot_connected().await;
    }

    pub async fn ws_disconnected(&mut self) {
        self.ws_connected = false;
        self.on_outgoing_offline().await;
        self.on_boot_disconnected().await;
        self.connect(self.cms_url.clone()).await;
    }

    pub(crate) async fn send_error(&mut self, uid: String, err: ProtocolError) {
        let err = CallError::new(uid, err);
        self.send_ws_msg(err.encode()).await;
    }

    pub async fn got_ws_msg(&mut self, msg: String) {
        self.heartbeat_activity().await;

        let res = OcppMessage::<ProtocolError>::decode(msg);
        match res {
            OcppMessage::Call(call) => match call.action.as_str() {
                "CancelReservation" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.cancel_reservation_ocpp(unique_id, req).await;
                    }).await;
                }
                "ChangeAvailability" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.change_availability_ocpp(unique_id, req).await;
                    }).await;
                }
                "ChangeConfiguration" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.change_configuration_ocpp(unique_id, req).await;
                    }).await;
                }
                "ClearCache" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.clear_cache_ocpp(unique_id, req).await;
                    }).await;
                }
                "ClearChargingProfile" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.clear_charging_profile_ocpp(unique_id, req).await;
                    }).await;
                }
                "DataTransfer" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.data_transfer_ocpp(unique_id, req).await;
                    }).await;
                }
                "GetCompositeSchedule" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.get_composite_schedule_ocpp(unique_id, req).await;
                    }).await;
                }
                "GetConfiguration" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.get_configuration_ocpp(unique_id, req).await;
                    }).await;
                }
                "GetDiagnostics" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.get_diagnostics_ocpp(unique_id, req).await;
                    }).await;
                }
                "GetLocalListVersion" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.get_local_list_version_ocpp(unique_id, req).await;
                    }).await;
                }
                "RemoteStartTransaction" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.remote_start_transaction_ocpp(unique_id, req).await;
                    }).await;
                }
                "RemoteStopTransaction" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.remote_stop_transaction_ocpp(unique_id, req).await;
                    }).await;
                }
                "ReserveNow" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.reserve_now_ocpp(unique_id, req).await;
                    }).await;
                }
                "Reset" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.reset_ocpp(unique_id, req).await;
                    }).await;
                }
                "SendLocalList" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.send_local_list_ocpp(unique_id, req).await;
                    }).await;
                }
                "SetChargingProfile" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.set_charging_profile_ocpp(unique_id, req).await;
                    }).await;
                }
                "TriggerMessage" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.trigger_message_ocpp(unique_id, req).await;
                    }).await;
                }
                "UnlockConnector" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.unlock_connector_ocpp(unique_id, req).await;
                    }).await;
                }
                "UpdateFirmware" => {
                    self.handle_call(call, |srv, unique_id, req| async {
                        srv.update_firmware_ocpp(unique_id, req).await;
                    }).await;
                }
                _ => {
                    self.send_error(call.unique_id, ProtocolError::NotSupported).await;
                }
            },
            OcppMessage::CallResponse(res) => {
                self.handle_call_response(Ok(res), true).await;
            }
            OcppMessage::Invalid(invalid) => {
                if let Some(uid) = invalid.unique_id {
                    self.send_error(uid, ProtocolError::FormationViolation).await;
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
    async fn handle_call<'a, T, H, Fut>(
        &'a mut self,
        call: Call,
        handler: H
    )
    where 
    T: serde::de::DeserializeOwned,
    H: FnOnce(&'a mut Self, String, T) -> Fut,
    Fut: core::future::Future<Output = ()> + 'a,
    {
        let uid = call.unique_id.clone();

        match serde_json::from_value::<T>(call.payload) {
            Ok(val) => {
                if self.is_registered(&call.action) {
                    handler(self, uid, val).await;
                } else {
                    self.send_error(uid, ProtocolError::SecurityError).await;
                }
            }
            Err(_) => {
                self.send_error(uid, ProtocolError::FormationViolation).await;
            }
        }
    }
}