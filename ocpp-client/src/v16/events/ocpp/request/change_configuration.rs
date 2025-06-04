use alloc::string::String;
use rocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::change_configuration::{ChangeConfigurationRequest, ChangeConfigurationResponse},
        types::ConfigurationStatus,
    },
};

use crate::v16::{cp::ChargePoint, interfaces::{ChargePointBackend, ChargePointInterface}, state_machine::config::OcppConfig};

macro_rules! gen_update_ocpp_match {
    ($this:ident, $key:expr, { $($key_str:literal => $field:ident),+ }, $raw:expr) => {
        match $key {
            $(
                $key_str => $this.config_update_helper(|s| (&mut s.configs.$field, &mut s.interface), $raw).await,
            )+
            _ => Err(ConfigurationStatus::NotSupported)
        }
    };
}

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn change_configuration_ocpp(
        &mut self,
        unique_id: String,
        req: ChangeConfigurationRequest,
    ) {
        let key = req.key.as_str();
        let value = req.value;
        let res = config_key_map!(gen_update_ocpp_match, self, key, value);
        let status = match res {
            Ok(t) => match t {
                true => ConfigurationStatus::RebootRequired,
                false => ConfigurationStatus::Accepted,
            },
            Err(e) => e,
        };

        let payload = ChangeConfigurationResponse { status };
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode()).await;
    }
    async fn config_update_helper<T>(
        &mut self,
        accessor: fn(&mut Self) -> (&mut OcppConfig<T>, &mut ChargePointBackend<I>),
        raw: String,
    ) -> Result<bool, ConfigurationStatus> {
        let (cfg_ref, db) = accessor(self);
        let new_val = (cfg_ref.parser_fn)(&raw)
            .ok_or(ConfigurationStatus::Rejected)?;

        if let Some(validator) = &cfg_ref.validator {
            validator(&new_val)
                .then_some(())
                .ok_or(ConfigurationStatus::Rejected)?;
        }
        cfg_ref.update_with_raw(new_val, raw, db).await;
        Ok(cfg_ref.reboot_required)
    }
}
