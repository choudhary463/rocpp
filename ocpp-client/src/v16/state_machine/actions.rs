use alloc::string::String;

use crate::v16::{
    cp::ChargePoint,
    interfaces::{ChargePointInterface, TimerId},
};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub async fn connect(&mut self, cms_url: String) {
        log::debug!("connect, cms_url: {}", cms_url);
        self.interface.interface.ws_connect(cms_url).await;
    }

    pub async fn send_ws_msg(&mut self, msg: String) {
        log::info!("[MSG_OUT] {}", msg);
        self.interface.interface.ws_send(msg).await;
    }

    pub async fn start_diagnostics_upload(&mut self, location: String, timeout: u64) {
        log::debug!("start upload, location: {} ,timeout: {}", location, timeout);
        self.interface
            .interface
            .diagnostics_upload(location, timeout)
            .await;
    }

    pub async fn download_firmware(&mut self, location: String) {
        log::debug!("download firmware, location: {}", location);
        self.interface.interface.firmware_download(location).await;
    }

    pub async fn install_firmware(&mut self) {
        log::debug!("install firmware");
        self.interface.interface.firmware_install().await;
    }

    pub async fn add_timeout(&mut self, timer_id: TimerId, timeout_secs: u64) {
        log::trace!(
            "add timeout, id: {:?}, deadline: {:?}",
            timer_id,
            timeout_secs
        );
        self.interface
            .interface
            .add_or_update_timeout(timer_id, timeout_secs)
            .await;
    }

    pub async fn remove_timeout(&mut self, timer_id: TimerId) {
        log::trace!("remove timeout, id: {:?}", timer_id);
        self.interface.interface.remove_timeout(timer_id).await;
    }

    pub fn soft_reset(&mut self) {
        log::warn!("soft reset");
        self.soft_reset_now = true;
    }
}
