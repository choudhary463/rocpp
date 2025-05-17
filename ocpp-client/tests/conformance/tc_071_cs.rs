use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use ocpp_client::v16::MeterDataType;
use ocpp_core::v16::{
    messages::{
        get_configuration::{GetConfigurationRequest, GetConfigurationResponse},
        meter_values::{MeterValuesRequest, MeterValuesResponse},
    },
    types::ReadingContext,
};

use crate::state::{
    reusable_states::{
        config_key_handler, get_sampled_value, validate_meter_values, AuthorizeState, BootState,
        ChargingState, ReusableState,
    },
    step::TestChain,
    ws_recv::AfterValidation,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let transaction_id = 1;
    let id_tag = format!("1234");
    let clock_aligned_data_interval = 4;
    let tol = 20;
    let count = 5;

    let chain = BootState::default(num_connectors)
        .get_test_chain()
        .call(GetConfigurationRequest {
            key: Some(vec!["MeterValuesAlignedData".into()]),
        })
        .await_ws_msg::<GetConfigurationResponse>()
        .done_custom(config_key_handler(
            "MeterValuesAlignedData",
            move |value, _readonly| {
                let value = match value {
                    Some(t) => t,
                    None => {
                        return AfterValidation::Failed(anyhow!(
                            "no value found for key MeterValuesAlignedData"
                        ))
                    }
                };
                let value = if value.is_empty() {
                    format!("Energy.Active.Import.Register")
                } else {
                    value
                };
                let measurands = match MeterDataType::parse_meter_data(&value) {
                    Ok(t) => t,
                    Err(e) => return AfterValidation::Failed(e),
                };
                let last_ts: Arc<Mutex<Option<DateTime<Utc>>>> = Arc::new(Mutex::new(None));
                let mut chain = TestChain::new();
                for id in 1..=num_connectors {
                    let name = format!("MeterValues_befor_transaction_{}", id);
                    let ms = measurands.clone();

                    chain = chain
                        .await_ws_msg::<MeterValuesRequest>()
                        .done_custom(move |t| {
                            let transaction_id = None;
                            let values = match get_sampled_value(&name, t, id, transaction_id) {
                                Ok(v) => v,
                                Err(e) => return AfterValidation::Failed(e),
                            };
                            let _ = match validate_meter_values(
                                &name,
                                values,
                                Some(ms.clone()),
                                ReadingContext::SampleClock,
                                None,
                                clock_aligned_data_interval,
                                tol,
                            ) {
                                Ok(ts) => ts,
                                Err(e) => return AfterValidation::Failed(e),
                            };
                            AfterValidation::NextDefault
                        })
                        .respond(MeterValuesResponse {});
                }
                chain = chain.any_order(num_connectors);

                chain = chain.merge(
                    AuthorizeState::default(num_connectors, connector_id, id_tag.clone())
                        .get_self_chain()
                        .merge(
                            ChargingState::default(
                                num_connectors,
                                connector_id,
                                transaction_id,
                                id_tag,
                            )
                            .get_self_chain(),
                        ),
                );

                for i in 1..=count {
                    for id in 1..=num_connectors {
                        let name = format!("MeterValues{}_{}", i, id);
                        let ms = measurands.clone();
                        let ts_handle = last_ts.clone();

                        chain = chain
                            .await_ws_msg::<MeterValuesRequest>()
                            .done_custom(move |t| {
                                let transaction_id = if id == connector_id {
                                    Some(transaction_id)
                                } else {
                                    None
                                };
                                let values = match get_sampled_value(&name, t, id, transaction_id) {
                                    Ok(v) => v,
                                    Err(e) => return AfterValidation::Failed(e),
                                };
                                let prev_ts = if id == connector_id {
                                    *ts_handle.lock().unwrap()
                                } else {
                                    None
                                };
                                let new_ts = match validate_meter_values(
                                    &name,
                                    values,
                                    Some(ms.clone()),
                                    ReadingContext::SampleClock,
                                    prev_ts,
                                    clock_aligned_data_interval,
                                    tol,
                                ) {
                                    Ok(ts) => ts,
                                    Err(e) => return AfterValidation::Failed(e),
                                };
                                if id == connector_id {
                                    *ts_handle.lock().unwrap() = Some(new_ts);
                                }
                                AfterValidation::NextDefault
                            })
                            .respond(MeterValuesResponse {});
                    }
                    chain = chain.any_order(num_connectors);
                }
                AfterValidation::NextCustom(chain.build())
            },
        ));

    chain
        .run(
            15,
            vec![(
                "ClockAlignedDataInterval",
                clock_aligned_data_interval.to_string().as_str(),
            )],
            None,
        )
        .await;
}
