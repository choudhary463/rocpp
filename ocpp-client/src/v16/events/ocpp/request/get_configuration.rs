use alloc::{string::{String, ToString}, vec::Vec};
use ocpp_core::{
    format::{
        frame::{CallError, CallResult},
        message::EncodeDecode,
    },
    v16::{
        messages::get_configuration::{GetConfigurationRequest, GetConfigurationResponse},
        protocol_error::ProtocolError,
        types::KeyValue,
    },
};

use crate::v16::{
    drivers::{database::Database, hardware_interface::HardwareInterface},
    state_machine::{config::OcppConfig},
    cp::core::ChargePointCore
};

macro_rules! gen_get_ocpp_match {
    ($this:ident, $key:expr, { $($key_str:literal => $field:ident),+ }) => {
        match $key {
            $(
                $key_str => Some($this.config_get_helper(|s| &s.configs.$field)),
            )+
            _ => None,
        }
    };
}

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn get_configuration_ocpp(&mut self, unique_id: String, req: GetConfigurationRequest) {
        if req.key.as_ref().map(|t| t.len()).unwrap_or(0)
            > self.configs.get_configuration_max_keys.value
        {
            let res = CallError::new(unique_id, ProtocolError::OccurrenceConstraintViolation);
            self.send_ws_msg(res.encode());
            return;
        }
        let mut configuration_key = Vec::new();
        let mut unknown_key = Vec::new();
        let keys = if req.key.as_ref().map(|t| t.is_empty()).unwrap_or(true) {
            self.db
                .get_all_config()
                .into_iter()
                .map(|t| t.0)
                .collect::<Vec<_>>()
        } else {
            req.key.unwrap()
        };
        for key in keys {
            let key = key.as_str();
            let res = config_key_map!(gen_get_ocpp_match, self, key);
            if let Some(value) = res {
                configuration_key.push(value);
            } else {
                unknown_key.push(key.to_string());
            }
        }
        let configuration_key = if configuration_key.is_empty() {
            None
        } else {
            Some(configuration_key)
        };
        let unknown_key = if unknown_key.is_empty() {
            None
        } else {
            Some(unknown_key)
        };
        let payload = GetConfigurationResponse {
            configuration_key,
            unknown_key,
        };
        let res = CallResult::new(unique_id, payload);

        self.send_ws_msg(res.encode());
    }
    fn config_get_helper<T>(&self, accessor: fn(&Self) -> &OcppConfig<T>) -> KeyValue {
        let cfg_ref = accessor(self);
        let value = if cfg_ref.read {
            Some(cfg_ref.raw.clone())
        } else {
            None
        };
        KeyValue {
            key: cfg_ref.key.clone(),
            readonly: !cfg_ref.write,
            value,
        }
    }
}
