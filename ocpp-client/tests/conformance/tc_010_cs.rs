use anyhow::anyhow;
use ocpp_core::v16::{
    messages::{
        authorize::{AuthorizeRequest, AuthorizeResponse},
        change_configuration::{ChangeConfigurationRequest, ChangeConfigurationResponse},
        get_configuration::{GetConfigurationRequest, GetConfigurationResponse},
        remote_start_transaction::{RemoteStartTransactionRequest, RemoteStartTransactionResponse},
        start_transaction::{StartTransactionRequest, StartTransactionResponse},
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
    },
    types::{
        AuthorizationStatus, ChargePointStatus, ConfigurationStatus, IdTagInfo,
        RemoteStartStopStatus,
    },
};

use crate::{
    state::{
        reusable_states::{config_key_handler, stop_transaction_chain, BootState, ReusableState},
        step::TestChain,
        ws_recv::AfterValidation,
    },
    test_chain,
};

fn get_authorize_remote_transaction_requests_true_chain(
    connector_id: usize,
    id_tag: String,
    id_tag_info: IdTagInfo,
    transaction_id: i32,
    authorize_remote_transaction_requests: bool,
) -> TestChain {
    let mut chain = test_chain!(
        TestChain::new(),
        plug(connector_id),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Preparing
        }),
        respond(StatusNotificationResponse {}),
        call(RemoteStartTransactionRequest {
            connector_id: Some(connector_id),
            id_tag: id_tag.clone(),
            charging_profile: None
        }),
        await_ws_msg(RemoteStartTransactionResponse {
            status: RemoteStartStopStatus::Accepted
        }),
    );

    if authorize_remote_transaction_requests {
        chain = test_chain!(
            chain,
            await_ws_msg(AuthorizeRequest {
                id_tag: id_tag.clone()
            }),
            respond(AuthorizeResponse {
                id_tag_info: id_tag_info.clone()
            }),
        );
    }

    test_chain!(
        chain,
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Charging
        }),
        respond(StatusNotificationResponse {}),
        await_ws_msg(StartTransactionRequest {
            connector_id: connector_id,
            id_tag: id_tag.clone(),
            reservation_id: None
        }),
        respond(StartTransactionResponse {
            id_tag_info: id_tag_info.clone(),
            transaction_id: transaction_id
        }),
        any_order(2),
    )
}

fn change_authorize_remote_transaction_requests(
    authorize_remote_transaction_requests: bool,
) -> TestChain {
    test_chain!(
        TestChain::new(),
        call(ChangeConfigurationRequest {
            key: "AuthorizeRemoteTxRequests".into(),
            value: authorize_remote_transaction_requests.to_string()
        }),
        await_ws_msg(ChangeConfigurationResponse {
            status: ConfigurationStatus::Accepted
        }),
    )
}

pub async fn run() {
    let num_connectors = 2;
    let connector_id = 1;
    let transaction_id1 = 1;
    let transaction_id2 = 2;
    let id_tag = format!("1234");
    let id_tag_info = IdTagInfo {
        expiry_date: None,
        parent_id_tag: None,
        status: AuthorizationStatus::Accepted,
    };

    let chain = test_chain!(
        BootState::default(num_connectors).get_test_chain(),
        call(GetConfigurationRequest {
            key: Some(vec!["AuthorizeRemoteTxRequests".into()])
        })
    );

    let chain = chain
        .await_ws_msg::<GetConfigurationResponse>()
        .done_custom(config_key_handler(
            "AuthorizeRemoteTxRequests",
            move |value, readonly| {
                let value = match value {
                    Some(t) => t,
                    None => {
                        return AfterValidation::Failed(anyhow!(
                            "no value found for key AuthorizeRemoteTxRequests"
                        ))
                    }
                };
                let val: bool = match value.parse() {
                    Ok(t) => t,
                    Err(e) => return AfterValidation::Failed(e.into()),
                };
                let chain = if !readonly {
                    get_authorize_remote_transaction_requests_true_chain(
                        connector_id,
                        id_tag.clone(),
                        id_tag_info.clone(),
                        transaction_id1,
                        val,
                    )
                    .merge(stop_transaction_chain(
                        connector_id,
                        id_tag.clone(),
                        transaction_id1,
                    ))
                    .merge(change_authorize_remote_transaction_requests(!val))
                    .merge(
                        get_authorize_remote_transaction_requests_true_chain(
                            connector_id,
                            id_tag,
                            id_tag_info,
                            transaction_id2,
                            !val,
                        ),
                    )
                } else {
                    get_authorize_remote_transaction_requests_true_chain(
                        connector_id,
                        id_tag.clone(),
                        id_tag_info.clone(),
                        transaction_id1,
                        val,
                    )
                };
                AfterValidation::NextCustom(chain.build())
            },
        ));

    chain.run(15, vec![], None).await;
}
