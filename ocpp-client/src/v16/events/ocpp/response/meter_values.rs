use rocpp_core::v16::messages::meter_values::MeterValuesResponse;

use crate::v16::{cp::{ChargePoint, OcppError}, interfaces::{ChargePointInterface, TimerId}, state_machine::transaction::{TransactionEvent, TransactionEventState}};


impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn meter_values_response(&mut self, res: Result<MeterValuesResponse, OcppError>) {
        let local_transaction_id = match &self.transacion_current_event {
            Some(TransactionEvent::Meter(t)) => t.local_transaction_id,
            _ => {
                unreachable!();
            }
        };
        match res {
            Ok(_) => {
                self.pop_event(local_transaction_id, None, None).await;
                self.process_transaction().await;
            }
            Err(e) => {
                log::error!("meter_values_response error: {:?}", e);
                if self.transaction_event_retries == self.configs.transaction_message_attempts.value
                {
                    self.pop_event(local_transaction_id, None, None).await;
                    self.process_transaction().await;
                } else {
                    self.add_timeout(
                        TimerId::Transaction,
                        self.configs.transaction_message_retry_interval.value
                            * self.transaction_event_retries,
                    ).await;
                    self.transaction_event_state = TransactionEventState::Sleeping;
                }
            }
        }
    }
}
