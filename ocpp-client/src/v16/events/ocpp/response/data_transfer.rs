use ocpp_core::v16::messages::data_transfer::DataTransferResponse;

use crate::v16::{
    interface::{Database, Secc},
    cp::{ChargePointCore, OcppError},
};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub(crate) fn _data_transfer_response(&mut self, res: Result<DataTransferResponse, OcppError>) {
        match res {
            Ok(_) => {}
            Err(e) => {
                log::error!("boot_notification_response error: {:?}", e);
            }
        }
    }
}
