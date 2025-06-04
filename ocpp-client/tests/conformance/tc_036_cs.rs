use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use rocpp_core::v16::{
    messages::meter_values::{MeterValuesRequest, MeterValuesResponse},
    types::ReadingContext,
};

use crate::{
    state::{
        reusable_states::{
            get_sampled_value, validate_meter_values, ChargingState, ConnectionState, ReusableState,
        },
        ws_recv::AfterValidation,
    },
    test_chain,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let transaction_id = 1;
    let id_tag = format!("1234");
    let base_dir = std::env::temp_dir();
    let db_dir = Some(base_dir.join("tc_036"));
    let meter_value_sample_interval = 5;
    let tol = 20;
    let count = 3;

    let mut chain = test_chain!(
        ChargingState::default(num_connectors, connector_id, transaction_id, id_tag)
            .get_test_chain(),
        close_connection(),
        await_disconnection(),
        await_timeout(),
        restore_connection(),
        merge(ConnectionState::default().get_test_chain())
    );

    let last_ts: Arc<Mutex<Option<DateTime<Utc>>>> = Arc::new(Mutex::new(None));
    for i in 1..=count {
        let name = format!("MeterValues{}", i);
        let ts_handle = last_ts.clone();

        chain = chain
            .await_ws_msg::<MeterValuesRequest>()
            .done_custom(move |t| {
                let values = match get_sampled_value(&name, t, connector_id, Some(transaction_id)) {
                    Ok(v) => v,
                    Err(e) => return AfterValidation::Failed(e),
                };
                let prev_ts = *ts_handle.lock().unwrap();
                let new_ts = match validate_meter_values(
                    &name,
                    values,
                    None,
                    ReadingContext::SamplePeriodic,
                    prev_ts,
                    meter_value_sample_interval,
                    tol,
                ) {
                    Ok(ts) => ts,
                    Err(e) => return AfterValidation::Failed(e),
                };
                *ts_handle.lock().unwrap() = Some(new_ts);
                AfterValidation::NextDefault
            })
            .respond(MeterValuesResponse {});
    }
    chain = chain.combine(count).with_timing(0, 20);

    chain
        .run(
            20,
            vec![(
                "MeterValueSampleInterval",
                meter_value_sample_interval.to_string().as_str(),
            )],
            db_dir,
        )
        .await;
}
