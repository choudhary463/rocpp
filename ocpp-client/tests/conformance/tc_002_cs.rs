use chrono::Utc;
use rocpp_core::v16::{
    messages::{
        boot_notification::{BootNotificationRequest, BootNotificationResponse},
        change_configuration::{ChangeConfigurationRequest, ChangeConfigurationResponse},
        get_configuration::{GetConfigurationRequest, GetConfigurationResponse},
        heart_beat::{HeartbeatRequest, HeartbeatResponse},
    },
    types::{ChargePointStatus, ConfigurationStatus, KeyValue, RegistrationStatus},
};

use crate::{
    harness::harness::get_cms_url,
    state::{reusable_states::get_all_connector_states, step::TestChain},
    test_chain,
};

pub async fn run() {
    let url = get_cms_url();
    let num_connectors = 2;

    let chain = test_chain!(
        TestChain::new(),
        await_connection(url.clone(), false),
        await_ws_msg(BootNotificationRequest {}),
        respond_with_now(BootNotificationResponse {
            current_time: Utc::now(),
            interval: 2,
            status: RegistrationStatus::Pending
        }),
        call(GetConfigurationRequest {
            key: Some(vec!["NumberOfConnectors".into()])
        }),
        await_ws_msg(GetConfigurationResponse {
            configuration_key: Some(vec![KeyValue {
                key: "NumberOfConnectors".into(),
                value: Some(num_connectors.to_string()),
                readonly: true
            }])
        }),
        call(ChangeConfigurationRequest {
            key: "AuthorizationCacheEnabled".into(),
            value: "false".into()
        }),
        await_ws_msg(ChangeConfigurationResponse {
            status: ConfigurationStatus::Accepted
        }),
        await_ws_msg(BootNotificationRequest {}),
        respond_with_now(BootNotificationResponse {
            current_time: Utc::now(),
            interval: 1,
            status: RegistrationStatus::Accepted
        }),
        merge(get_all_connector_states(vec![
            ChargePointStatus::Available;
            num_connectors
        ])),
        await_ws_msg(HeartbeatRequest {}),
        respond_with_now(HeartbeatResponse {
            current_time: Utc::now()
        }),
        with_timing(1000, 20),
        await_ws_msg(HeartbeatRequest {}),
        respond_with_now(HeartbeatResponse {
            current_time: Utc::now()
        }),
        with_timing(1000, 20)
    );

    chain.run(15, vec![], None).await;
}
