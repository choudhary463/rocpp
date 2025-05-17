use anyhow::anyhow;
use chrono::{DateTime, Utc};
use ocpp_client::v16::MeterDataType;
use ocpp_core::v16::{
    messages::{
        authorize::{AuthorizeRequest, AuthorizeResponse},
        boot_notification::{BootNotificationRequest, BootNotificationResponse},
        get_configuration::GetConfigurationResponse,
        meter_values::MeterValuesRequest,
        start_transaction::{StartTransactionRequest, StartTransactionResponse},
        status_notification::{StatusNotificationRequest, StatusNotificationResponse},
        stop_transaction::{StopTransactionRequest, StopTransactionResponse},
    },
    protocol_error::ProtocolError,
    types::{
        AuthorizationStatus, ChargePointStatus, IdTagInfo, Location, Measurand, MeterValue,
        ReadingContext, RegistrationStatus, ValueFormat,
    },
};

use crate::{harness::harness::get_cms_url, test_chain};

use super::{step::TestChain, ws_recv::AfterValidation};

pub trait ReusableState {
    fn get_self_chain(&self) -> TestChain;
    fn get_test_chain(self) -> TestChain;
}

pub struct ConnectionState {
    url: String,
    with_disconnection: bool,
}

impl ConnectionState {
    pub fn default() -> Self {
        Self {
            url: get_cms_url(),
            with_disconnection: false,
        }
    }
    pub fn with_disconnection(mut self) -> Self {
        self.with_disconnection = true;
        self
    }
}

impl ReusableState for ConnectionState {
    fn get_self_chain(&self) -> TestChain {
        TestChain::new().await_connection(self.url.clone(), self.with_disconnection)
    }
    fn get_test_chain(self) -> TestChain {
        self.get_self_chain()
    }
}

pub struct BootState {
    conn: ConnectionState,
    interval: u64,
    expected_connector_state: Vec<ChargePointStatus>,
}

impl BootState {
    pub fn default(num_connectors: usize) -> Self {
        Self {
            conn: ConnectionState::default(),
            interval: 1000,
            expected_connector_state: vec![ChargePointStatus::Available; num_connectors],
        }
    }
    pub fn custom_expected_connector_state(
        expected_connector_state: Vec<ChargePointStatus>,
    ) -> Self {
        Self {
            conn: ConnectionState::default(),
            interval: 1000,
            expected_connector_state: expected_connector_state,
        }
    }
    pub fn with_state(mut self, connector_id: usize, state: ChargePointStatus) -> Self {
        self.expected_connector_state[connector_id - 1] = state;
        self
    }
}

pub fn get_all_connector_states(expected_connector_state: Vec<ChargePointStatus>) -> TestChain {
    let mut res = TestChain::new();
    let len = expected_connector_state.len();
    for (index, status) in expected_connector_state.clone().into_iter().enumerate() {
        let connector_id = index + 1;
        res = res
            .await_ws_msg::<StatusNotificationRequest>()
            .check_eq(&connector_id, |t| &t.connector_id)
            .check_eq(&status, |t| &t.status)
            .done()
            .respond(StatusNotificationResponse {})
    }
    res = res.any_order(len);
    res
}

impl ReusableState for BootState {
    fn get_self_chain(&self) -> TestChain {
        TestChain::new()
            .await_ws_msg::<BootNotificationRequest>()
            .done()
            .respond_with_now(BootNotificationResponse {
                current_time: Utc::now(),
                interval: self.interval,
                status: RegistrationStatus::Accepted,
            })
            .merge(get_all_connector_states(
                self.expected_connector_state.clone(),
            ))
    }
    fn get_test_chain(self) -> TestChain {
        let self_chain = self.get_self_chain();
        self.conn.get_test_chain().merge(self_chain)
    }
}

pub struct AuthorizeState {
    boot: BootState,
    connector_id: usize,
    id_tag: String,
    id_tag_info: IdTagInfo,
}

impl AuthorizeState {
    pub fn default(num_connectors: usize, connector_id: usize, id_tag: String) -> Self {
        AuthorizeState {
            boot: BootState::default(num_connectors),
            connector_id,
            id_tag,
            id_tag_info: IdTagInfo {
                expiry_date: None,
                parent_id_tag: None,
                status: AuthorizationStatus::Accepted,
            },
        }
    }
    pub fn custom_with_default_boot(
        num_connectors: usize,
        connector_id: usize,
        id_tag: String,
        id_tag_info: IdTagInfo,
    ) -> Self {
        AuthorizeState {
            boot: BootState::default(num_connectors),
            connector_id,
            id_tag,
            id_tag_info,
        }
    }
}

impl ReusableState for AuthorizeState {
    fn get_self_chain(&self) -> TestChain {
        test_chain!(
            TestChain::new(),
            present_id_tag(self.connector_id, self.id_tag.clone()),
            await_ws_msg(AuthorizeRequest {
                id_tag: self.id_tag
            }),
            respond(AuthorizeResponse {
                id_tag_info: self.id_tag_info.clone()
            }),
            await_ws_msg(StatusNotificationRequest {
                connector_id: self.connector_id,
                status: ChargePointStatus::Preparing
            }),
            respond(StatusNotificationResponse {})
        )
    }
    fn get_test_chain(self) -> TestChain {
        let self_chain = self.get_self_chain();
        self.boot.get_test_chain().merge(self_chain)
    }
}

pub struct ChargingState {
    auth: AuthorizeState,
    transaction_id: i32,
}

impl ChargingState {
    pub fn default(
        num_connectors: usize,
        connector_id: usize,
        transaction_id: i32,
        id_tag: String,
    ) -> Self {
        Self {
            auth: AuthorizeState::default(num_connectors, connector_id, id_tag),
            transaction_id,
        }
    }
    pub fn custom(auth: AuthorizeState, transaction_id: i32) -> Self {
        Self {
            auth,
            transaction_id,
        }
    }
}

impl ReusableState for ChargingState {
    fn get_self_chain(&self) -> TestChain {
        test_chain!(
            TestChain::new(),
            plug(self.auth.connector_id),
            await_ws_msg(StatusNotificationRequest {
                connector_id: self.auth.connector_id,
                status: ChargePointStatus::Charging
            }),
            respond(StatusNotificationResponse {}),
            await_ws_msg(StartTransactionRequest {
                connector_id: self.auth.connector_id,
                id_tag: self.auth.id_tag
            }),
            respond(StartTransactionResponse {
                id_tag_info: self.auth.id_tag_info.clone(),
                transaction_id: self.transaction_id
            }),
            any_order(2)
        )
    }
    fn get_test_chain(self) -> TestChain {
        let self_chain = self.get_self_chain();
        self.auth.get_test_chain().merge(self_chain)
    }
}

pub struct IdTagCachedState {
    boot: BootState,
    connector_id: usize,
    id_tag: String,
    id_tag_info: IdTagInfo,
}

impl IdTagCachedState {
    pub fn default(
        num_connectors: usize,
        connector_id: usize,
        id_tag: String,
        id_tag_info: IdTagInfo,
    ) -> Self {
        Self {
            boot: BootState::default(num_connectors),
            connector_id,
            id_tag,
            id_tag_info,
        }
    }
}

pub fn stop_transaction_chain(
    connector_id: usize,
    id_tag: String,
    transaction_id: i32,
) -> TestChain {
    test_chain!(
        TestChain::new(),
        present_id_tag(connector_id, id_tag.clone()),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Finishing
        }),
        respond(StatusNotificationResponse {}),
        await_ws_msg(StopTransactionRequest {
            transaction_id: transaction_id,
            id_tag: Some(id_tag)
        }),
        respond(StopTransactionResponse { id_tag_info: None }),
        any_order(2),
        unplug(connector_id),
        await_ws_msg(StatusNotificationRequest {
            connector_id: connector_id,
            status: ChargePointStatus::Available
        }),
        respond(StatusNotificationResponse {})
    )
}

impl ReusableState for IdTagCachedState {
    fn get_self_chain(&self) -> TestChain {
        let mut chain = test_chain!(
            TestChain::new(),
            present_id_tag(self.connector_id, self.id_tag.clone()),
            await_ws_msg(AuthorizeRequest {
                id_tag: self.id_tag
            }),
            respond(AuthorizeResponse {
                id_tag_info: self.id_tag_info.clone()
            }),
        );
        if self.id_tag_info.is_valid(Some(Utc::now())) {
            chain = test_chain!(
                chain,
                await_ws_msg(StatusNotificationRequest {
                    connector_id: self.connector_id,
                    status: ChargePointStatus::Preparing
                }),
                respond(StatusNotificationResponse {}),
                await_ws_msg(StatusNotificationRequest {
                    connector_id: self.connector_id,
                    status: ChargePointStatus::Available
                }),
                respond(StatusNotificationResponse {}),
            )
        }
        chain
    }
    fn get_test_chain(self) -> TestChain {
        let self_chain = self.get_self_chain();
        self.boot.get_test_chain().merge(self_chain)
    }
}

pub fn config_key_handler<B>(
    key_name: &'static str,
    handler: B,
) -> impl FnOnce(&Result<GetConfigurationResponse, ProtocolError>) -> AfterValidation
where
    B: FnOnce(Option<String>, bool) -> AfterValidation,
{
    move |res| match res {
        Err(e) => AfterValidation::Failed(anyhow!("RPC error fetching `{}`: {:?}", key_name, e)),
        Ok(resp) => {
            let keys = match resp.configuration_key.as_ref() {
                Some(t) => t,
                None => {
                    return AfterValidation::Failed(anyhow!(
                        "expected exactly one `{}` entry, got nothing",
                        key_name
                    ))
                }
            };
            if keys.len() != 1 {
                return AfterValidation::Failed(anyhow!(
                    "expected exactly one `{}` entry, got {}",
                    key_name,
                    keys.len()
                ));
            }
            let cfg = keys.into_iter().next().unwrap();
            if !resp
                .unknown_key
                .as_ref()
                .map(|u| u.is_empty())
                .unwrap_or(true)
            {
                return AfterValidation::Failed(anyhow!(
                    "`unknown_key` not empty for `{}`: {:?}",
                    key_name,
                    resp.unknown_key
                ));
            }

            if cfg.key != key_name {
                return AfterValidation::Failed(anyhow!(
                    "expected key `{}`, found `{}`",
                    key_name,
                    cfg.key
                ));
            }

            handler(cfg.value.clone(), cfg.readonly)
        }
    }
}

pub fn get_sampled_value(
    step: &str,
    meter_values: &Result<MeterValuesRequest, ProtocolError>,
    connector_id: usize,
    transaction_id: Option<i32>,
) -> Result<MeterValue, anyhow::Error> {
    let meter = match meter_values {
        Ok(t) => t,
        Err(e) => return Err(anyhow!("exected payload for {}, found err {:?}", step, e)),
    };
    if meter.connector_id != connector_id {
        return Err(anyhow!(
            "expected connector_id for {}: {}, found {}",
            step,
            connector_id,
            meter.connector_id
        ));
    }
    if meter.transaction_id != transaction_id {
        return Err(anyhow!(
            "expected transaction_id for {}:  {:?}, found {:?}",
            step,
            transaction_id,
            meter.transaction_id
        ));
    }
    if meter.meter_value.len() != 1 {
        return Err(anyhow!("expected meter values length 1"));
    }
    Ok(meter.meter_value[0].clone())
}

pub fn validate_meter_values(
    step: &str,
    meter_values: MeterValue,
    measurands: Option<Vec<MeterDataType>>,
    context: ReadingContext,
    last_time: Option<DateTime<Utc>>,
    meter_value_sample_interval: u64,
    tol: u64,
) -> Result<DateTime<Utc>, anyhow::Error> {
    if let Some(last_time) = last_time {
        let diff = meter_values.timestamp - last_time;
        if diff.num_seconds() < 0
            || meter_value_sample_interval.abs_diff(diff.num_seconds() as u64) > tol
        {
            return Err(anyhow!("incorrect time difference between consecutive frames is not {}s±{}ms, prev received at {}, next at {}", meter_value_sample_interval, tol, last_time, meter_values.timestamp));
        }
    }
    let mut received_measurands = Vec::new();
    for value in meter_values.sampled_value {
        let ctx = value
            .context
            .clone()
            .unwrap_or(ReadingContext::SamplePeriodic);
        if ctx != context {
            return Err(anyhow!(
                "expected context for {}: {:?}, found {:?}",
                step,
                context,
                ctx
            ));
        }
        if value
            .format
            .as_ref()
            .map(|t| *t != ValueFormat::Raw)
            .unwrap_or(false)
        {
            return Err(anyhow!(
                "found meter value format as SignedData in {}, metervalue: {:?}",
                step,
                value
            ));
        }
        if value
            .location
            .as_ref()
            .map(|t| *t == Location::Ev)
            .unwrap_or(false)
        {
            if value
                .measurand
                .as_ref()
                .map(|t| *t != Measurand::SoC)
                .unwrap_or(true)
            {
                return Err(anyhow!(
                    "location = EV found while measurand != SoC in {}",
                    step
                ));
            }
        }
        received_measurands.push(MeterDataType {
            measurand: value
                .measurand
                .unwrap_or(Measurand::EnergyActiveImportRegister),
            phase: value.phase,
        });
    }
    let check = |a: &[MeterDataType], b: &[MeterDataType]| -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut b_remaining = b.to_vec();
        for item in a {
            if let Some(pos) = b_remaining.iter().position(|x| x == item) {
                b_remaining.remove(pos);
            } else {
                return false;
            }
        }

        b_remaining.is_empty()
    };
    let measurands = match measurands {
        Some(t) => t,
        None => return Ok(meter_values.timestamp),
    };
    if !check(&measurands, &received_measurands) {
        return Err(anyhow!(
            "expected measurands for {}: {:?}, found {:?}",
            step,
            measurands,
            received_measurands
        ));
    }
    Ok(meter_values.timestamp)
}
