use ocpp_core::v16::messages::stop_transaction::StopTransactionResponse;

use crate::v16::{
    interface::{Database, Secc},
    services::timeout::TimerId,
    state_machine::{
        core::{ChargePointCore, OcppError},
        transaction::{TransactionEvent, TransactionEventState},
    },
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn stop_transaction_response(&mut self, res: Result<StopTransactionResponse, OcppError>) {
        let (local_transaction_id, id_tag) = match self.transaction_queue.front() {
            Some(TransactionEvent::Stop(t)) => (t.local_transaction_id, t.id_tag.clone()),
            _ => {
                unreachable!();
            }
        };
        let meter_tx = self
            .transaction_stop_meter_map
            .get(&local_transaction_id)
            .map(|f| f.len())
            .unwrap_or(0);
        match res {
            Ok(t) => {
                if let (Some(id_tag), Some(info)) = (id_tag, t.id_tag_info) {
                    self.update_cache(id_tag, info);
                }
                self.pop_event(Some(local_transaction_id), None, Some(meter_tx));
            }
            Err(e) => {
                log::error!("stop_transaction_response error: {:?}", e);
                if self.transaction_event_retries == self.configs.transaction_message_attempts.value
                {
                    self.pop_event(Some(local_transaction_id), None, Some(meter_tx));
                } else {
                    self.add_timeout(
                        TimerId::Transaction,
                        self.configs.transaction_message_retry_interval.value
                            * self.transaction_event_retries,
                    );
                    self.transaction_event_state = TransactionEventState::Sleeping
                }
            }
        }
    }
}
