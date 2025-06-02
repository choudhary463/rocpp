use ocpp_core::v16::messages::data_transfer::DataTransferResponse;

use crate::v16::{
    drivers::{database::Database, hardware_interface::HardwareInterface},
    cp::core::{ChargePointCore, OcppError},
};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn _data_transfer_response(&mut self, res: Result<DataTransferResponse, OcppError>) {
        match res {
            Ok(_) => {}
            Err(e) => {
                log::error!("boot_notification_response error: {:?}", e);
            }
        }
    }
}
