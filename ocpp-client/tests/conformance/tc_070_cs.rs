use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use rocpp_client::v16::MeterDataType;
use rocpp_core::v16::{
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
    ws_recv::AfterValidation,
};

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let transaction_id = 1;
    let id_tag = format!("1234");
    let meter_value_sample_interval = 4;
    let tol = 20;
    let count = 5;

    let chain = BootState::default(num_connectors)
        .get_test_chain()
        .call(GetConfigurationRequest {
            key: Some(vec!["MeterValuesSampledData".into()]),
        })
        .await_ws_msg::<GetConfigurationResponse>()
        .done_custom(config_key_handler(
            "MeterValuesSampledData",
            move |value, _readonly| {
                let value = match value {
                    Some(t) => t,
                    None => {
                        return AfterValidation::Failed(anyhow!(
                            "no value found for key MeterValuesSampledData"
                        ))
                    }
                };
                let value = if value.is_empty() {
                    format!("Energy.Active.Import.Register")
                } else {
                    value
                };
                let measurands = match MeterDataType::parse_meter_data(&value) {
                    Some(t) => t,
                    None => return AfterValidation::Failed(anyhow!("measurand parse error")),
                };
                let last_ts: Arc<Mutex<Option<DateTime<Utc>>>> = Arc::new(Mutex::new(None));
                let mut chain =
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
                        );
                for i in 1..=count {
                    let name = format!("MeterValues{}", i);
                    let ms = measurands.clone();
                    let ts_handle = last_ts.clone();

                    chain = chain
                        .await_ws_msg::<MeterValuesRequest>()
                        .done_custom(move |t| {
                            let values = match get_sampled_value(
                                &name,
                                t,
                                connector_id,
                                Some(transaction_id),
                            ) {
                                Ok(v) => v,
                                Err(e) => return AfterValidation::Failed(e),
                            };
                            let prev_ts = *ts_handle.lock().unwrap();
                            let new_ts = match validate_meter_values(
                                &name,
                                values,
                                Some(ms.clone()),
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
                AfterValidation::NextCustom(chain.build())
            },
        ));

    chain
        .run(
            15,
            vec![(
                "MeterValueSampleInterval",
                meter_value_sample_interval.to_string().as_str(),
            )],
            None,
        )
        .await;
}
