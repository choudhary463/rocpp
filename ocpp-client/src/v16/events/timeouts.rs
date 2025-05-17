use ocpp_core::{
    format::error::GenericError,
    v16::types::{FirmwareStatus, ReadingContext},
};

use crate::v16::{
    interface::{Database, Secc},
    services::timeout::TimerId,
    state_machine::{
        boot::BootState,
        call::OutgoingCallState,
        connector::{ConnectorState, StatusNotificationState},
        core::{ChargePointCore, OcppError},
        diagnostics::DiagnosticsState,
        firmware::{FirmwareDownloadInfo, FirmwareState},
        heartbeat::HeartbeatState,
        meter::MeterDataKind,
        transaction::TransactionEventState,
    },
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn handle_timeout(&mut self, id: TimerId) {
        match id {
            TimerId::Boot => {
                match &self.boot_state {
                    BootState::Sleeping => {
                        self.send_boot_notification();
                    }
                    _ => {
                        // boot expiry received while not in WaitingForResponse state
                        unreachable!();
                    }
                }
            }
            TimerId::Heartbeat => {
                match &self.heartbeat_state {
                    HeartbeatState::Sleeping => {
                        self.send_heartbeat();
                    }
                    _ => {
                        // heartbeat expiry received while not in Sleeping state
                        unreachable!();
                    }
                }
            }
            TimerId::Call => match &self.outgoing_call_state {
                OutgoingCallState::WaitingForResponse { .. } => {
                    self.handle_call_response(Err(OcppError::Other(GenericError::TimeOut)), true);
                }
                _ => {
                    unreachable!();
                }
            },
            TimerId::StatusNotification(connector_id) => {
                match &self.connector_status_notification_state[connector_id] {
                    StatusNotificationState::Stabilizing(_) => {
                        self.send_status_notification(connector_id);
                    }
                    _ => {
                        unreachable!();
                    }
                }
            }
            TimerId::Authorize(connector_id) => match &self.connector_state[connector_id] {
                ConnectorState::Authorized { .. } => {
                    self.change_connector_state(connector_id, ConnectorState::idle());
                }
                _ => {
                    unreachable!();
                }
            },
            TimerId::Reservation(connector_id) => match &self.connector_state[connector_id] {
                ConnectorState::Reserved {
                    is_plugged,
                    reservation_id,
                    ..
                } => {
                    let is_plugged = *is_plugged;
                    self.remove_reservation(connector_id, *reservation_id);
                    if is_plugged {
                        self.change_connector_state(connector_id, ConnectorState::plugged());
                    } else {
                        self.change_connector_state(connector_id, ConnectorState::idle());
                    }
                }
                _ => {
                    unreachable!();
                }
            },
            TimerId::Diagnostics => {
                if let DiagnosticsState::Uploading(mut t) =
                    std::mem::replace(&mut self.diagnostics_state, DiagnosticsState::Idle)
                {
                    t.retry_left -= 1;
                    self.cancel_diagnostics_upload();
                    self.diagnostics_state = DiagnosticsState::Uploading(t);
                    self.try_diagnostrics_upload();
                } else {
                    unreachable!();
                }
            }
            TimerId::Firmware => {
                match std::mem::replace(&mut self.firmware_state, FirmwareState::Idle) {
                    FirmwareState::New(t) => {
                        self.send_firmware_status_notification(FirmwareStatus::Downloading);
                        self.try_firmware_download(FirmwareDownloadInfo {
                            retry_left: t.retries.unwrap_or(1),
                            retry_interval: t.retry_interval.unwrap_or(0),
                            location: t.location,
                        });
                    }
                    FirmwareState::DownloadSleep(t) => {
                        self.send_firmware_status_notification(FirmwareStatus::Downloading);
                        self.try_firmware_download(t);
                    }
                    _ => {
                        unreachable!();
                    }
                }
            }
            TimerId::MeterAligned => {
                for connector_id in 0..self.configs.number_of_connectors.value {
                    let mut local_tx = None;
                    if let Some((local_transaction_id, _)) =
                        self.active_local_transactions[connector_id]
                    {
                        local_tx = Some(local_transaction_id);
                        self.add_stop_transaction_sampled_data(
                            connector_id,
                            local_transaction_id,
                            MeterDataKind::StopTxnAligned,
                            ReadingContext::SampleClock,
                        );
                    }
                    self.add_meter_event(
                        connector_id,
                        local_tx,
                        MeterDataKind::MeterValuesAligned,
                        ReadingContext::SampleClock,
                    );
                }
                self.set_aligned_meter_sleep_state();
            }
            TimerId::MeterSampled(connector_id) => {
                let local_transaction_id = self.active_local_transactions[connector_id].unwrap().0;
                self.add_meter_event(
                    connector_id,
                    Some(local_transaction_id),
                    MeterDataKind::MeterValuesSampled,
                    ReadingContext::SamplePeriodic,
                );
                self.add_stop_transaction_sampled_data(
                    connector_id,
                    local_transaction_id,
                    MeterDataKind::StopTxnSampled,
                    ReadingContext::SamplePeriodic,
                );
                self.set_sampled_meter_sleep_state(connector_id);
            }
            TimerId::Transaction => match &self.transaction_event_state {
                TransactionEventState::Sleeping => {
                    self.transaction_event_state = TransactionEventState::Idle;
                    self.process_transaction();
                }
                _ => {
                    unreachable!();
                }
            },
        }
    }
}
