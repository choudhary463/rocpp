use rocpp_core::v16::messages::data_transfer::DataTransferResponse;

use crate::v16::{cp::{ChargePoint, OcppError}, interfaces::ChargePointInterface};


impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) fn _data_transfer_response(&mut self, res: Result<DataTransferResponse, OcppError>) {
        match res {
            Ok(_) => {}
            Err(e) => {
                log::error!("boot_notification_response error: {:?}", e);
            }
        }
    }
}
