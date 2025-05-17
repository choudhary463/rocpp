use anyhow::anyhow;
use ocpp_core::v16::{
    messages::{
        change_configuration::{ChangeConfigurationRequest, ChangeConfigurationResponse},
        get_configuration::{GetConfigurationRequest, GetConfigurationResponse},
    },
    protocol_error::ProtocolError,
    types::ConfigurationStatus,
};

use crate::state::{
    reusable_states::{BootState, ReusableState},
    ws_recv::AfterValidation,
};

pub async fn run() {
    let num_connectors = 2;

    let chain = BootState::default(num_connectors)
        .get_test_chain()
        .call(ChangeConfigurationRequest {
            key: "MeterValueSampleInterval".into(),
            value: "10".into(),
        })
        .await_ws_msg::<ChangeConfigurationResponse>()
        .check_eq(&ConfigurationStatus::Accepted, |t| &t.status)
        .done()
        .call(GetConfigurationRequest {
            key: Some(vec!["MeterValueSampleInterval".into()]),
        })
        .await_ws_msg::<GetConfigurationResponse>()
        .done_custom(move |t| {
            let get_single =
                |t: &GetConfigurationResponse, key: &str| -> Result<(Option<String>, bool), &str> {
                    if t.configuration_key
                        .as_ref()
                        .map(|t| t.len() != 1)
                        .unwrap_or(false)
                    {
                        return Err("expected configuration_keys len = 1");
                    }
                    if t.unknown_key
                        .as_ref()
                        .map(|t| !t.is_empty())
                        .unwrap_or(false)
                    {
                        return Err("unknown_key is not empty");
                    }
                    let k = t.configuration_key.as_ref().unwrap()[0].clone();
                    if k.key.as_str() != key {
                        return Err("key mismatch");
                    }
                    Ok((k.value, k.readonly))
                };
            let check = |t: &Result<GetConfigurationResponse, ProtocolError>| {
                let res = t
                    .as_ref()
                    .map_err(|_| "expected CallResult, found CallError")?;
                let (value, _) = get_single(res, "MeterValueSampleInterval")?;
                let value = value.ok_or("expected value")?;
                if value != "10".to_string() {
                    return Err("Invalid MeterValueSampleInterval");
                }
                Ok(())
            };
            let res: Result<(), &str> = check(t);
            match res {
                Ok(_) => AfterValidation::NextDefault,
                Err(e) => AfterValidation::Failed(anyhow!(e)),
            }
        });

    chain.run(15, vec![], None).await;
}
