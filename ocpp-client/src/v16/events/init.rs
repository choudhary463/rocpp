use crate::v16::{cp::ChargePointCore, interface::{Database, Secc}};

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub(crate) fn init_helper(&mut self) {
        self.handle_unfinished_transactions();
    }
}
