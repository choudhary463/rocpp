use alloc::{string::{String, ToString}, vec::Vec};
use rocpp_core::{
    format::{
        error::GenericError,
        frame::Call,
        message::{CallResponse, EncodeDecode},
    },
    v16::{protocol_error::ProtocolError, types::ResetType},
};
use serde::Serialize;

use crate::v16::{cp::{ChargePoint, OcppError}, interfaces::{ChargePointInterface, TimerId}};

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

impl core::fmt::Display for CallAction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn on_outgoing_offline(&mut self) {
        self.handle_call_response(Err(OcppError::Other(GenericError::Offline)), false).await;
        let drained: Vec<_> = self.pending_calls.drain(..).collect();
        for (_, action) in drained {
            self.dispatch_response(action, Err(OcppError::Other(GenericError::Offline))).await;
        }
    }

    pub(crate) async fn enqueue_call<T: Serialize>(&mut self, action: CallAction, payload: T) {
        let call = Call {
            unique_id: self.get_uuid(),
            action: action.to_string(),
            payload: serde_json::to_value(payload).unwrap(),
        };
        self.pending_calls.push_back((call, action));
        self.process_call().await;
    }

    pub(crate) async fn process_call(&mut self) {
        if let OutgoingCallState::Idle = self.outgoing_call_state {
            if let Some((call, action)) = self.pending_calls.pop_front() {
                self.send_ws_msg(call.encode()).await;
                self.outgoing_call_state = OutgoingCallState::WaitingForResponse {
                    unique_id: call.unique_id,
                    action,
                };
                self.add_timeout(TimerId::Call, self.call_timeout).await;
            }
        }
    }

    pub(crate) async fn handle_call_response(
        &mut self,
        res: Result<CallResponse<ProtocolError>, OcppError>,
        check_next: bool,
    ) {
        if let Some(res) = self.match_call_uid(res) {
            self.remove_timeout(TimerId::Call).await;
            if let OutgoingCallState::WaitingForResponse { action, .. } =
                core::mem::replace(&mut self.outgoing_call_state, OutgoingCallState::Idle)
            {
                self.dispatch_response(action, res).await;
                if check_next {
                    self.process_call().await;
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
    async fn dispatch_response(&mut self, action: CallAction, res: Result<serde_json::Value, OcppError>) {
        match action {
            CallAction::BootNotification => {
                self.boot_notification_response(Self::parse_response(res)).await
            }
            CallAction::Heartbeat => self.heartbeat_response(Self::parse_response(res)).await,
            CallAction::Authorize => self.authorized_response(Self::parse_response(res)).await,
            CallAction::StatusNotification => {
                self.status_notification_response(Self::parse_response(res))
            }
            CallAction::StartTransaction => {
                self.start_transaction_response(Self::parse_response(res)).await
            }
            CallAction::MeterValues => self.meter_values_response(Self::parse_response(res)).await,
            CallAction::StopTransaction => {
                self.stop_transaction_response(Self::parse_response(res)).await
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
