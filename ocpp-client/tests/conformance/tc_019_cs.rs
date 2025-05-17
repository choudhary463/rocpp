use anyhow::anyhow;
use ocpp_core::v16::{
    messages::get_configuration::{GetConfigurationRequest, GetConfigurationResponse},
    protocol_error::ProtocolError,
};

use crate::state::{
    reusable_states::{config_key_handler, BootState, ReusableState},
    ws_recv::AfterValidation,
};

pub async fn run() {
    let num_connectors = 2;

    let chain = BootState::default(num_connectors)
        .get_test_chain()
        .call(GetConfigurationRequest {
            key: Some(vec!["SupportedFeatureProfiles".into()]),
        })
        .await_ws_msg::<GetConfigurationResponse>()
        .done_custom(config_key_handler(
            "SupportedFeatureProfiles",
            move |value, _readonly| {
                let value = match value {
                    Some(t) => t,
                    None => {
                        return AfterValidation::Failed(anyhow!(
                            "no value found for key SupportedFeatureProfiles"
                        ))
                    }
                };
                let valid = [
                    "Core",
                    "FirmwareManagement",
                    "LocalAuthListManagement",
                    "Reservation",
                    "SmartCharging",
                    "RemoteTrigger",
                ];
                for s in value.split(',') {
                    if !valid.contains(&s.trim()) {
                        return AfterValidation::Failed(anyhow!("invalid profile: {}", s.trim()));
                    }
                }
                AfterValidation::NextDefault
            },
        ))
        .call(GetConfigurationRequest { key: None })
        .await_ws_msg::<GetConfigurationResponse>()
        .done_custom(move |t| {
            let check = |t: &Result<GetConfigurationResponse, ProtocolError>| {
                let res = t
                    .as_ref()
                    .map_err(|_| "expected CallResult, found CallError".to_string())?;
                if res
                    .unknown_key
                    .as_ref()
                    .map(|t| !t.is_empty())
                    .unwrap_or(false)
                {
                    return Err("unknown_key is not empty".to_string());
                }
                let all = res
                    .configuration_key
                    .as_ref()
                    .ok_or("Empty configuration_key")?;
                let check_key = |key: &str| -> Result<(), String> {
                    all.iter()
                        .find(|t| t.key == key)
                        .ok_or(format!("{} key not found", key))?;
                    Ok(())
                };
                let check_mutability = |key: &str, is_read_only: bool| -> Result<(), String> {
                    let f = all
                        .iter()
                        .find(|t| t.key == key)
                        .ok_or(format!("{} key not found", key))?;
                    if f.readonly != is_read_only {
                        return Err(format!("invalid mutability of key {}", key));
                    }
                    Ok(())
                };
                check_key("AuthorizeRemoteTxRequests")?;
                check_mutability("ClockAlignedDataInterval", false)?;
                check_mutability("ConnectionTimeOut", false)?;
                check_mutability("ConnectorPhaseRotation", false)?;
                check_mutability("GetConfigurationMaxKeys", true)?;
                check_mutability("HeartbeatInterval", false)?;
                check_mutability("LocalAuthorizeOffline", false)?;
                check_mutability("LocalPreAuthorize", false)?;
                check_mutability("MeterValuesAlignedData", false)?;
                check_mutability("MeterValuesSampledData", false)?;
                check_mutability("MeterValueSampleInterval", false)?;
                check_mutability("NumberOfConnectors", true)?;
                check_mutability("ResetRetries", false)?;
                check_mutability("StopTransactionOnInvalidId", false)?;
                check_mutability("StopTxnAlignedData", false)?;
                check_mutability("StopTxnSampledData", false)?;
                check_mutability("SupportedFeatureProfiles", true)?;
                check_mutability("TransactionMessageAttempts", false)?;
                check_mutability("TransactionMessageRetryInterval", false)?;
                check_key("UnlockConnectorOnEVSideDisconnect")?;
                check_mutability("LocalAuthListEnabled", false)?;
                check_mutability("LocalAuthListMaxLength", true)?;
                check_mutability("SendLocalListMaxLength", true)?;

                Ok(())
            };
            let res: Result<(), String> = check(t);
            match res {
                Ok(_) => AfterValidation::NextDefault,
                Err(e) => AfterValidation::Failed(anyhow!(e)),
            }
        });

    chain.run(15, vec![], None).await;
}
