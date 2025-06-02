use core::str::FromStr;

use alloc::{boxed::Box, format, string::{String, ToString}, vec::Vec};

use ocpp_core::v16::types::{Measurand, Phase};

use crate::v16::drivers::{database::{ChargePointStorage, Database}, hardware_interface::MeterDataType};

pub(crate) struct OcppConfig<T> {
    pub key: String,
    pub raw: String,
    pub value: T,
    pub read: bool,
    pub write: bool,
    pub reboot_required: bool,
    pub parser_fn: fn(&str) -> Option<T>,
    pub format_fn: fn(&T) -> String,
    pub validator: Option<Box<dyn Fn(&T) -> bool + Send>>,
}

impl<T> OcppConfig<T> {
    pub fn new() -> Self
    where
        T: Default,
    {
        Self {
            key: String::new(),
            raw: String::new(),
            value: T::default(),
            read: false,
            write: false,
            reboot_required: false,
            parser_fn: |_| None,
            format_fn: |_| String::new(),
            validator: None,
        }
    }
    pub fn with_std(mut self) -> Self
    where
        T: ToString + FromStr,
        <T as FromStr>::Err: core::error::Error + Send + Sync + 'static,
    {
        self.parser_fn = |s| s.parse().ok();
        self.format_fn = |v| v.to_string();
        self
    }
    pub fn with_parse(mut self, parser_fn: fn(&str) -> Option<T>) -> Self {
        self.parser_fn = parser_fn;
        self
    }
    pub fn with_format_fn(mut self, format_fn: fn(&T) -> String) -> Self {
        self.format_fn = format_fn;
        self
    }
    pub fn read(mut self) -> Self {
        self.read = true;
        self
    }
    pub fn write(mut self) -> Self {
        self.write = true;
        self
    }
    pub fn reboot_required(mut self) -> Self {
        self.reboot_required = true;
        self
    }
    pub fn update<D: Database>(&mut self, value: T, database: &mut ChargePointStorage<D>) {
        let raw = (self.format_fn)(&value);
        database.db_update_config(self.key.clone(), raw.clone());
        self.raw = raw;
        self.value = value;
    }
    pub fn update_with_raw<D: Database>(
        &mut self,
        value: T,
        raw: String,
        database: &mut ChargePointStorage<D>,
    ) {
        database.db_update_config(self.key.clone(), raw.clone());
        if !self.reboot_required {
            self.raw = raw;
            self.value = value;
        }
    }
}

macro_rules! config_key_map {
    ($macro:ident, $this:ident, $key:expr $(, $args:tt)*) => {
        $macro!($this, $key, {
            "HeartbeatInterval" => heartbeat_interval,
            "MinimumStatusDuration" => minimum_status_duration,
            "AuthorizationCacheEnabled" => authorization_cache_enabled,
            "LocalAuthListEnabled" => local_auth_list_enabled,
            "LocalAuthListMaxLength" => local_auth_list_max_length,
            "SendLocalListMaxLength" => send_local_list_max_length,
            "AllowOfflineTxForUnknownId" => allow_offline_transaction_for_unknown_id,
            "LocalAuthorizeOffline" => local_authorize_offline,
            "LocalPreAuthorize" => local_pre_authorize,
            "NumberOfConnectors" => number_of_connectors,
            "ConnectionTimeOut" => connection_time_out,
            "StopTransactionOnEVSideDisconnect" => stop_transaction_on_evside_disconnect,
            "MeterValueSampleInterval" => meter_value_sample_interval,
            "ClockAlignedDataInterval" => clock_aligned_data_interval,
            "MeterValuesSampledData" => meter_values_sampled_data,
            "StopTxnSampledData" => stop_transaction_sampled_data,
            "MeterValuesAlignedData" => meter_values_aligned_data,
            "StopTxnAlignedData" => stop_transaction_aligned_data,
            "MeterValuesSampledDataMaxLength" => meter_values_sampled_data_max_length,
            "StopTxnSampledDataMaxLength" => stop_transaction_sampled_data_max_length,
            "MeterValuesAlignedDataMaxLength" => meter_values_aligned_data_max_length,
            "StopTxnAlignedDataMaxLength" => stop_transaction_aligned_data_max_length,
            "StopTransactionOnInvalidId" => stop_transaction_on_invalid_id,
            "TransactionMessageAttempts" => transaction_message_attempts,
            "TransactionMessageRetryInterval" => transaction_message_retry_interval,
            "AuthorizeRemoteTxRequests" => authorize_remote_transaction_requests,
            "ConnectorPhaseRotation" => connector_phase_rotation,
            "ResetRetries" => reset_retries,
            "GetConfigurationMaxKeys" => get_configuration_max_keys,
            "SupportedFeatureProfiles" => supported_feature_profiles,
            "UnlockConnectorOnEVSideDisconnect" => unlock_connector_on_evside_disconnect
        } $(, $args)*)
    };
}

// pub(crate) use config_key_map as config_key_map;

macro_rules! gen_update_match {
    ($this:ident, $key:expr, { $($key_str:literal => $field:ident),+ }, $raw:expr) => {
        match $key {
            $(
                $key_str => $this.init_config(|s| &mut s.$field, $key.to_string(), $raw),
            )+
            _ => {
                unreachable!();
            }
        }
    };
}

pub(crate) struct OcppConfigs {
    pub heartbeat_interval: OcppConfig<u64>,
    pub minimum_status_duration: OcppConfig<u64>,
    pub authorization_cache_enabled: OcppConfig<bool>,
    pub local_auth_list_enabled: OcppConfig<bool>,
    pub local_auth_list_max_length: OcppConfig<usize>,
    pub send_local_list_max_length: OcppConfig<usize>,
    pub allow_offline_transaction_for_unknown_id: OcppConfig<bool>,
    pub local_authorize_offline: OcppConfig<bool>,
    pub local_pre_authorize: OcppConfig<bool>,
    pub number_of_connectors: OcppConfig<usize>,
    pub connection_time_out: OcppConfig<u64>,
    pub stop_transaction_on_evside_disconnect: OcppConfig<bool>,
    pub meter_value_sample_interval: OcppConfig<u64>,
    pub clock_aligned_data_interval: OcppConfig<u64>,
    pub meter_values_sampled_data: OcppConfig<Vec<MeterDataType>>,
    pub stop_transaction_sampled_data: OcppConfig<Vec<MeterDataType>>,
    pub meter_values_aligned_data: OcppConfig<Vec<MeterDataType>>,
    pub stop_transaction_aligned_data: OcppConfig<Vec<MeterDataType>>,
    pub meter_values_sampled_data_max_length: OcppConfig<usize>,
    pub stop_transaction_sampled_data_max_length: OcppConfig<usize>,
    pub meter_values_aligned_data_max_length: OcppConfig<usize>,
    pub stop_transaction_aligned_data_max_length: OcppConfig<usize>,
    pub stop_transaction_on_invalid_id: OcppConfig<bool>,
    pub transaction_message_attempts: OcppConfig<u64>,
    pub transaction_message_retry_interval: OcppConfig<u64>,
    pub authorize_remote_transaction_requests: OcppConfig<bool>,
    pub connector_phase_rotation: OcppConfig<String>,
    pub reset_retries: OcppConfig<usize>,
    pub get_configuration_max_keys: OcppConfig<usize>,
    pub supported_feature_profiles: OcppConfig<String>,
    pub unlock_connector_on_evside_disconnect: OcppConfig<bool>,
}

impl OcppConfigs {
    fn new() -> Self {
        Self {
            heartbeat_interval: OcppConfig::<u64>::new().with_std().read().write(),
            minimum_status_duration: OcppConfig::<u64>::new().with_std().read().write(),
            authorization_cache_enabled: OcppConfig::<bool>::new().with_std().read().write(),
            local_auth_list_enabled: OcppConfig::<bool>::new().with_std().read().write(),
            local_auth_list_max_length: OcppConfig::<usize>::new().with_std().read(),
            send_local_list_max_length: OcppConfig::<usize>::new().with_std().read(),
            allow_offline_transaction_for_unknown_id: OcppConfig::<bool>::new()
                .with_std()
                .read()
                .write(),
            local_authorize_offline: OcppConfig::<bool>::new().with_std().read().write(),
            local_pre_authorize: OcppConfig::<bool>::new().with_std().read().write(),
            number_of_connectors: OcppConfig::<usize>::new().with_std().read(),
            connection_time_out: OcppConfig::<u64>::new().with_std().read().write(),
            stop_transaction_on_evside_disconnect: OcppConfig::<bool>::new()
                .with_std()
                .read()
                .write(),
            meter_value_sample_interval: OcppConfig::<u64>::new().with_std().read().write(),
            clock_aligned_data_interval: OcppConfig::<u64>::new()
                .with_std()
                .read()
                .write()
                .reboot_required(),
            meter_values_sampled_data: OcppConfig::<Vec<MeterDataType>>::new()
                .with_parse(MeterDataType::parse_meter_data)
                .with_format_fn(MeterDataType::format_meter_data)
                .read()
                .write(),
            stop_transaction_sampled_data: OcppConfig::<Vec<MeterDataType>>::new()
                .with_parse(MeterDataType::parse_meter_data)
                .with_format_fn(MeterDataType::format_meter_data)
                .read()
                .write(),
            meter_values_aligned_data: OcppConfig::<Vec<MeterDataType>>::new()
                .with_parse(MeterDataType::parse_meter_data)
                .with_format_fn(MeterDataType::format_meter_data)
                .read()
                .write(),
            stop_transaction_aligned_data: OcppConfig::<Vec<MeterDataType>>::new()
                .with_parse(MeterDataType::parse_meter_data)
                .with_format_fn(MeterDataType::format_meter_data)
                .read()
                .write(),
            meter_values_sampled_data_max_length: OcppConfig::<usize>::new().with_std().read(),
            stop_transaction_sampled_data_max_length: OcppConfig::<usize>::new().with_std().read(),
            meter_values_aligned_data_max_length: OcppConfig::<usize>::new().with_std().read(),
            stop_transaction_aligned_data_max_length: OcppConfig::<usize>::new().with_std().read(),
            stop_transaction_on_invalid_id: OcppConfig::<bool>::new().with_std().read().write(),
            transaction_message_attempts: OcppConfig::<u64>::new().with_std().read().write(),
            transaction_message_retry_interval: OcppConfig::<u64>::new().with_std().read().write(),
            authorize_remote_transaction_requests: OcppConfig::<bool>::new()
                .with_std()
                .read()
                .write(),
            connector_phase_rotation: OcppConfig::<String>::new().with_std().read().write(),
            reset_retries: OcppConfig::<usize>::new().with_std().read().write(),
            get_configuration_max_keys: OcppConfig::<usize>::new().with_std().read(),
            supported_feature_profiles: OcppConfig::<String>::new().with_std().read(),
            unlock_connector_on_evside_disconnect: OcppConfig::<bool>::new().with_std().read(),
        }
    }
    pub fn build(db_configs: Vec<(String, String)>) -> Self {
        let mut config = Self::new();
        for (key, value) in db_configs {
            let key = key.as_str();
            config_key_map!(gen_update_match, config, key, value)
        }
        config
    }
    fn init_config<T>(
        &mut self,
        accessor: fn(&mut Self) -> &mut OcppConfig<T>,
        key: String,
        raw: String,
    ) {
        let cfg_ref = accessor(self);

        let actual = (cfg_ref.parser_fn)(&raw).unwrap();
        cfg_ref.key = key;
        cfg_ref.raw = raw;
        cfg_ref.value = actual;
    }
}

impl MeterDataType {
    pub fn parse_meter_data(s: &str) -> Option<Vec<MeterDataType>> {
        s.split(',')
            .map(|token| {
                let token = token.trim();

                if let Some(idx) = token.rfind('.') {
                    let (left, right) = token.split_at(idx);
                    let measurand = serde_json::from_str::<Measurand>(&format!("\"{}\"", left));

                    let phase = serde_json::from_str::<Phase>(&format!("\"{}\"", &right[1..]));

                    match (measurand, phase) {
                        (Ok(measurand), Ok(phase)) => Some(MeterDataType {
                            measurand,
                            phase: Some(phase),
                        }),
                        _ => {
                            let measurand =
                                serde_json::from_str::<Measurand>(&format!("\"{}\"", token)).ok()?;
                            Some(MeterDataType {
                                measurand,
                                phase: None,
                            })
                        }
                    }
                } else {
                    let measurand = serde_json::from_str::<Measurand>(&format!("\"{}\"", token)).ok()?;
                    Some(MeterDataType {
                        measurand,
                        phase: None,
                    })
                }
            })
            .collect()
    }
    fn format_meter_data(v: &Vec<MeterDataType>) -> String {
        v.iter()
            .map(|data| {
                let m = &data.measurand;
                let p = &data.phase;
                let meas = serde_json::to_string(m)
                    .unwrap()
                    .trim_matches('"')
                    .to_string();
                match p {
                    Some(ph) => {
                        let phase_str = serde_json::to_string(ph).unwrap();
                        let phase_trimmed = phase_str.trim_matches('"');
                        format!("{meas}.{phase_trimmed}")
                    }
                    None => meas,
                }
            })
            .collect::<Vec<_>>()
            .join(",")
    }
}
