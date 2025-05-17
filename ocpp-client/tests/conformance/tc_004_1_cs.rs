use crate::state::reusable_states::{ChargingState, ReusableState};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let transaction_id = 1;
    let id_tag = format!("1234");

    let chain = ChargingState::default(num_connectors, connector_id, transaction_id, id_tag)
        .get_test_chain();

    chain.run(15, vec![], None).await;
}
