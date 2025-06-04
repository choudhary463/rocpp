use rocpp_core::v16::messages::stop_transaction::StopTransactionResponse;

use crate::v16::{cp::{ChargePoint, OcppError}, interfaces::{ChargePointInterface, TimerId}, state_machine::transaction::{TransactionEvent, TransactionEventState}};


impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn stop_transaction_response(&mut self, res: Result<StopTransactionResponse, OcppError>) {
        let (local_transaction_id, id_tag) = match &self.transacion_current_event {
            Some(TransactionEvent::Stop(t)) => (t.local_transaction_id, t.id_tag.clone()),
            _ => {
                unreachable!();
            }
        };
        let meter_tx = self
            .transaction_stop_meter_val_count
            .get(&local_transaction_id)
            .map(|f| *f)
            .unwrap_or(0);
        match res {
            Ok(t) => {
                if let (Some(id_tag), Some(info)) = (id_tag, t.id_tag_info) {
                    self.update_cache(id_tag, info).await;
                }
                self.pop_event(Some(local_transaction_id), None, Some(meter_tx)).await;
                self.process_transaction().await;
            }
            Err(e) => {
                log::error!("stop_transaction_response error: {:?}", e);
                if self.transaction_event_retries == self.configs.transaction_message_attempts.value
                {
                    self.pop_event(Some(local_transaction_id), None, Some(meter_tx)).await;
                    self.process_transaction().await;
                } else {
                    self.add_timeout(
                        TimerId::Transaction,
                        self.configs.transaction_message_retry_interval.value
                            * self.transaction_event_retries,
                    ).await;
                    self.transaction_event_state = TransactionEventState::Sleeping
                }
            }
        }
    }
}
