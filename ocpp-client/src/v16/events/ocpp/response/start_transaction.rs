use ocpp_core::v16::messages::start_transaction::StartTransactionResponse;

use crate::v16::{
    cp::core::{ChargePointCore, OcppError}, drivers::{database::Database, hardware_interface::HardwareInterface, timers::TimerId}, state_machine::transaction::{TransactionEvent, TransactionEventState}
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn start_transaction_response(&mut self, res: Result<StartTransactionResponse, OcppError>) {
        let (local_transaction_id, id_tag) = match self.transaction_queue.front() {
            Some(TransactionEvent::Start(t)) => (t.local_transaction_id, t.id_tag.clone()),
            _ => {
                unreachable!();
            }
        };
        match res {
            Ok(t) => {
                if !t.id_tag_info.is_valid(self.get_time()) {
                    self.deauthorize_transaction(local_transaction_id);
                }
                self.update_cache(id_tag, t.id_tag_info);
                self.pop_event(Some(local_transaction_id), Some(t.transaction_id), None);
            }
            Err(e) => {
                log::error!("start_transaction_response error: {:?}", e);
                if self.transaction_event_retries == self.configs.transaction_message_attempts.value
                {
                    self.pop_event(Some(local_transaction_id), None, None);
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
