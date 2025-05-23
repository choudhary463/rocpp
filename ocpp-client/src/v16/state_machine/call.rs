use ocpp_core::{
    format::{
        error::GenericError,
        frame::Call,
        message::{CallResponse, EncodeDecode},
    },
    v16::{protocol_error::ProtocolError, types::ResetType},
};
use serde::Serialize;

use crate::v16::{
    interface::{Database, Secc},
    services::timeout::TimerId,
    state_machine::core::ChargePointCore,
};

use super::core::OcppError;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub(crate) enum CallAction {
    BootNotification,
    Heartbeat,
    Authorize,
    StatusNotification,
    StartTransaction,
    MeterValues,
    StopTransaction,
    DiagnosticsStatusNotification,
    FirmwareStatusNotification,
}

impl std::fmt::Display for CallAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CallAction::BootNotification => "BootNotification",
            CallAction::Heartbeat => "Heartbeat",
            CallAction::Authorize => "Authorize",
            CallAction::StatusNotification => "StatusNotification",
            CallAction::StartTransaction => "StartTransaction",
            CallAction::MeterValues => "MeterValues",
            CallAction::StopTransaction => "StopTransaction",
            CallAction::DiagnosticsStatusNotification => "DiagnosticsStatusNotification",
            CallAction::FirmwareStatusNotification => "FirmwareStatusNotification"
        };
        write!(f, "{s}")
    }
}


#[derive(PartialEq)]
pub(crate) enum OutgoingCallState {
    Idle,
    WaitingForResponse {
        unique_id: String,
        action: CallAction,
    },
}

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn on_outgoing_offline(&mut self) {
        self.handle_call_response(Err(OcppError::Other(GenericError::Offline)), false);
        let drained: Vec<_> = self.pending_calls.drain(..).collect();
        for (_, action) in drained {
            self.dispatch_response(action, Err(OcppError::Other(GenericError::Offline)));
        }
    }

    pub fn enqueue_call<T: Serialize>(&mut self, action: CallAction, payload: T) {
        let call = Call {
            unique_id: uuid::Uuid::new_v4().to_string(),
            action: action.to_string(),
            payload: serde_json::to_value(payload).unwrap(),
        };
        self.pending_calls.push_back((call, action));
        self.process_call();
    }

    pub fn process_call(&mut self) {
        if let OutgoingCallState::Idle = self.outgoing_call_state {
            if let Some((call, action)) = self.pending_calls.pop_front() {
                self.send_ws_msg(call.encode());
                self.outgoing_call_state = OutgoingCallState::WaitingForResponse {
                    unique_id: call.unique_id,
                    action,
                };
                self.add_timeout(TimerId::Call, self.call_timeout);
            }
        }
    }

    pub fn handle_call_response(
        &mut self,
        res: Result<CallResponse<ProtocolError>, OcppError>,
        check_next: bool,
    ) {
        if let Some(res) = self.match_call_uid(res) {
            self.remove_timeout(TimerId::Call);
            if let OutgoingCallState::WaitingForResponse { action, .. } =
                std::mem::replace(&mut self.outgoing_call_state, OutgoingCallState::Idle)
            {
                self.dispatch_response(action, res);
                if check_next {
                    self.process_call();
                }
            }
        }
        if self.outgoing_call_state == OutgoingCallState::Idle {
            if let Some(ResetType::Soft) = self.pending_reset {
                self.soft_reset();
            }
        }
    }

    fn parse_response<T: serde::de::DeserializeOwned>(
        res: Result<serde_json::Value, OcppError>,
    ) -> Result<T, OcppError> {
        res.map(|f| {
            serde_json::from_value::<T>(f).map_err(|_| OcppError::Other(GenericError::ParsingError))
        })
        .and_then(|r| r)
    }
    fn dispatch_response(&mut self, action: CallAction, res: Result<serde_json::Value, OcppError>) {
        match action {
            CallAction::BootNotification => {
                self.boot_notification_response(Self::parse_response(res))
            }
            CallAction::Heartbeat => self.heartbeat_response(Self::parse_response(res)),
            CallAction::Authorize => self.authorized_response(Self::parse_response(res)),
            CallAction::StatusNotification => {
                self.status_notification_response(Self::parse_response(res))
            }
            CallAction::StartTransaction => {
                self.start_transaction_response(Self::parse_response(res))
            }
            CallAction::MeterValues => self.meter_values_response(Self::parse_response(res)),
            CallAction::StopTransaction => {
                self.stop_transaction_response(Self::parse_response(res))
            }
            CallAction::DiagnosticsStatusNotification => {
                self.diagnostics_status_notification_response(Self::parse_response(res))
            }
            CallAction::FirmwareStatusNotification => {
                self.firmware_status_notification_response(Self::parse_response(res))
            }
        }
    }

    fn match_call_uid(
        &self,
        res: Result<CallResponse<ProtocolError>, OcppError>,
    ) -> Option<Result<serde_json::Value, OcppError>> {
        match &self.outgoing_call_state {
            OutgoingCallState::Idle => None,
            OutgoingCallState::WaitingForResponse { unique_id, .. } => match res {
                Ok(CallResponse::CallResult(msg)) if msg.unique_id == *unique_id => {
                    Some(Ok(msg.payload))
                }
                Ok(CallResponse::CallError(err)) if err.unique_id == *unique_id => {
                    Some(Err(OcppError::Protocol(err.error_code)))
                }
                Err(e) => Some(Err(e)),
                _ => None,
            },
        }
    }
}
