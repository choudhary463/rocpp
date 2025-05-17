use anyhow::anyhow;
use ocpp_core::v16::{
    messages::{
        authorize::{AuthorizeRequest, AuthorizeResponse},
        change_configuration::{ChangeConfigurationRequest, ChangeConfigurationResponse},
        get_configuration::{GetConfigurationRequest, GetConfigurationResponse},
        remote_start_transaction::{RemoteStartTransactionRequest, RemoteStartTransactionResponse},
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
    },
    types::{
        AuthorizationStatus, ChargePointStatus, ConfigurationStatus, IdTagInfo,
        RemoteStartStopStatus,
    },
};

use crate::{
    state::{
        reusable_states::{config_key_handler, BootState, ReusableState},
        step::TestChain,
        ws_recv::AfterValidation,
    },
    test_chain,
};

fn get_authorize_remote_transaction_requests_chain(
    connector_id: usize,
    id_tag: String,
    id_tag_info: IdTagInfo,
    authorize_remote_transaction_requests: bool,
) -> TestChain {
    let mut res = test_chain!(
        TestChain::new(),
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
        res = test_chain!(
            res,
            await_ws_msg(AuthorizeRequest {
                id_tag: id_tag.clone()
            }),
            respond(AuthorizeResponse {
                id_tag_info: id_tag_info.clone()
            }),
        );
    }

    let res = test_chain!(
        res,
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Preparing
        }),
        respond(StatusNotificationResponse {}),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Available
        }),
        respond(StatusNotificationResponse {}),
    );

    res
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
                    get_authorize_remote_transaction_requests_chain(
                        connector_id,
                        id_tag.clone(),
                        id_tag_info.clone(),
                        val,
                    )
                    .merge(change_authorize_remote_transaction_requests(!val))
                    .merge(get_authorize_remote_transaction_requests_chain(
                        connector_id,
                        id_tag,
                        id_tag_info,
                        !val,
                    ))
                } else {
                    get_authorize_remote_transaction_requests_chain(
                        connector_id,
                        id_tag.clone(),
                        id_tag_info.clone(),
                        val,
                    )
                };
                AfterValidation::NextCustom(chain.build())
            },
        ));

    chain.run(15, vec![("ConnectionTimeOut", "4")], None).await;
}
