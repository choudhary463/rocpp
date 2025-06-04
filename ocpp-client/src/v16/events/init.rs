use crate::v16::{interfaces::ChargePointInterface, ChargePoint};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn init(&mut self) {
        self.connect(self.cms_url.clone()).await;
    }
}