use crate::v16::{cp::core::ChargePointCore, drivers::{database::Database, hardware_interface::HardwareInterface}};

impl<D: Database, H: HardwareInterface> ChargePointCore<D, H> {
    pub(crate) fn init_helper(&mut self) {
        self.handle_unfinished_transactions();
    }
}
