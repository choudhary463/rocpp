use ocpp_core::v16::{
    messages::change_availability::{ChangeAvailabilityRequest, ChangeAvailabilityResponse},
    types::{AvailabilityStatus, AvailabilityType, ChargePointStatus},
};

use crate::{
    state::reusable_states::{get_all_connector_states, BootState, ReusableState},
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 0;

    let base_dir = std::env::temp_dir();
    let db_dir = Some(base_dir.join("tc_034"));

    let mut chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(ChangeAvailabilityRequest {
            connector_id,
            kind: AvailabilityType::Inoperative
        }),
        await_ws_msg(ChangeAvailabilityResponse {
            status: AvailabilityStatus::Accepted
        })
    );
    chain = chain.merge(get_all_connector_states(vec![
        ChargePointStatus::Unavailable;
        num_connectors
    ]));
    chain = test_chain!(
        chain,
        cut_power(),
        await_hard_reset(),
        spawn_new(15, vec![], db_dir.clone(), false),
        merge(
            BootState::custom_expected_connector_state(vec![
                ChargePointStatus::Unavailable;
                num_connectors
            ])
            .get_test_chain()
        )
    );

    chain.run(15, vec![], db_dir).await;
}
