use rocpp_core::{
    format::error::GenericError,
    v16::types::{FirmwareStatus, ReadingContext},
};

use crate::v16::{
    cp::{ChargePoint, OcppError},
    interfaces::{ChargePointInterface, TimerId},
    state_machine::{
        boot::BootState,
        call::OutgoingCallState,
        connector::{ConnectorState, StatusNotificationState},
        firmware::{FirmwareDownloadInfo, FirmwareState},
        heartbeat::HeartbeatState,
        meter::MeterDataKind,
        transaction::TransactionEventState,
    },
};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub async fn handle_timeout(&mut self, id: TimerId) {
        self.remove_timeout(id.clone()).await;
        match id {
            TimerId::Boot => {
                match &self.boot_state {
                    BootState::Sleeping => {
                        self.send_boot_notification().await;
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
                        self.send_heartbeat().await;
                    }
                    _ => {
                        // heartbeat expiry received while not in Sleeping state
                        unreachable!();
                    }
                }
            }
            TimerId::Call => match &self.outgoing_call_state {
                OutgoingCallState::WaitingForResponse { .. } => {
                    self.handle_call_response(Err(OcppError::Other(GenericError::TimeOut)), true)
                        .await;
                }
                _ => {
                    unreachable!();
                }
            },
            TimerId::StatusNotification(connector_id) => {
                match &self.connector_status_notification_state[connector_id] {
                    StatusNotificationState::Stabilizing(_) => {
                        self.send_status_notification(connector_id).await;
                    }
                    _ => {
                        unreachable!();
                    }
                }
            }
            TimerId::Authorize(connector_id) => match &self.connector_state[connector_id] {
                ConnectorState::Authorized { .. } => {
                    self.change_connector_state(connector_id, ConnectorState::idle())
                        .await;
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
                    self.remove_reservation(connector_id, *reservation_id).await;
                    if is_plugged {
                        self.change_connector_state(connector_id, ConnectorState::plugged())
                            .await;
                    } else {
                        self.change_connector_state(connector_id, ConnectorState::idle())
                            .await;
                    }
                }
                _ => {
                    unreachable!();
                }
            },
            TimerId::Firmware => {
                match core::mem::replace(&mut self.firmware_state, FirmwareState::Idle) {
                    FirmwareState::New(t) => {
                        self.send_firmware_status_notification(FirmwareStatus::Downloading)
                            .await;
                        self.try_firmware_download(FirmwareDownloadInfo {
                            retry_left: t.retries.unwrap_or(1),
                            retry_interval: t.retry_interval.unwrap_or(0),
                            location: t.location,
                        })
                        .await;
                    }
                    FirmwareState::DownloadSleep(t) => {
                        self.send_firmware_status_notification(FirmwareStatus::Downloading)
                            .await;
                        self.try_firmware_download(t).await;
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
                        )
                        .await;
                    }
                    self.add_meter_event(
                        connector_id,
                        local_tx,
                        MeterDataKind::MeterValuesAligned,
                        ReadingContext::SampleClock,
                    )
                    .await;
                }
                self.set_aligned_meter_sleep_state().await;
            }
            TimerId::MeterSampled(connector_id) => {
                let local_transaction_id = self.active_local_transactions[connector_id].unwrap().0;
                self.add_meter_event(
                    connector_id,
                    Some(local_transaction_id),
                    MeterDataKind::MeterValuesSampled,
                    ReadingContext::SamplePeriodic,
                )
                .await;
                self.add_stop_transaction_sampled_data(
                    connector_id,
                    local_transaction_id,
                    MeterDataKind::StopTxnSampled,
                    ReadingContext::SamplePeriodic,
                )
                .await;
                self.set_sampled_meter_sleep_state(connector_id).await;
            }
            TimerId::Transaction => match &self.transaction_event_state {
                TransactionEventState::Sleeping => {
                    self.transaction_event_state = TransactionEventState::Idle;
                    self.process_transaction().await;
                }
                _ => {
                    unreachable!();
                }
            },
        }
    }
}
