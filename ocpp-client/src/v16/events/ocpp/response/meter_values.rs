use ocpp_core::v16::messages::meter_values::MeterValuesResponse;

use crate::v16::{
    interface::{Database, Secc, TimerId},
    state_machine::{
        transaction::{TransactionEvent, TransactionEventState},
    },
    cp::{ChargePointCore, OcppError},
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub(crate) fn meter_values_response(&mut self, res: Result<MeterValuesResponse, OcppError>) {
        let local_transaction_id = match self.transaction_queue.front() {
            Some(TransactionEvent::Meter(t)) => t.local_transaction_id,
            _ => {
                unreachable!();
            }
        };
        match res {
            Ok(_) => {
                self.pop_event(local_transaction_id, None, None);
            }
            Err(e) => {
                log::error!("meter_values_response error: {:?}", e);
                if self.transaction_event_retries == self.configs.transaction_message_attempts.value
                {
                    self.pop_event(local_transaction_id, None, None);
                } else {
                    self.add_timeout(
                        TimerId::Transaction,
                        self.configs.transaction_message_retry_interval.value
                            * self.transaction_event_retries,
                    );
                    self.transaction_event_state = TransactionEventState::Sleeping;
                }
            }
        }
    }
}
