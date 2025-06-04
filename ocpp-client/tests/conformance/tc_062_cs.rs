use rocpp_core::v16::{
    messages::data_transfer::{DataTransferRequest, DataTransferResponse},
    types::DataTransferStatus,
};

use crate::{
    state::reusable_states::{BootState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;

    let mut chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(DataTransferRequest {
            vendor_id: format!("vendor"),
            message_id: None,
            data: None
        })
    );
    chain = chain
        .await_ws_msg::<DataTransferResponse>()
        .check(|f| {
            (f.status == DataTransferStatus::Accepted)
                .then(|| format!("DataTransferStatus::Accepted was not expected"))
        })
        .done();
    chain.run(15, vec![], None).await;
}
