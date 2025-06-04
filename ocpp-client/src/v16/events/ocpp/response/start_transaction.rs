use rocpp_core::v16::messages::start_transaction::StartTransactionResponse;

use crate::v16::{cp::{ChargePoint, OcppError}, interfaces::{ChargePointInterface, TimerId}, state_machine::transaction::{TransactionEvent, TransactionEventState}};


impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn start_transaction_response(&mut self, res: Result<StartTransactionResponse, OcppError>) {
        let (local_transaction_id, id_tag) = match &self.transacion_current_event {
            Some(TransactionEvent::Start(t)) => (t.local_transaction_id, t.id_tag.clone()),
            _ => {
                unreachable!();
            }
        };
        match res {
            Ok(t) => {
                let is_valid = t.id_tag_info.is_valid(self.get_time().await);
                self.update_cache(id_tag, t.id_tag_info).await;
                self.pop_event(Some(local_transaction_id), Some(t.transaction_id), None).await;
                self.process_transaction().await;
                if !is_valid {
                    self.deauthorize_transaction(local_transaction_id).await;
                }
            }
            Err(e) => {
                log::error!("start_transaction_response error: {:?}", e);
                if self.transaction_event_retries == self.configs.transaction_message_attempts.value
                {
                    self.pop_event(Some(local_transaction_id), None, None).await;
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
