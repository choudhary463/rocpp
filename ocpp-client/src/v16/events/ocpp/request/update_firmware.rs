use alloc::string::String;
use rocpp_core::{
    format::{frame::CallResult, message::EncodeDecode},
    v16::{
        messages::update_firmware::{UpdateFirmwareRequest, UpdateFirmwareResponse},
        types::FirmwareStatus,
    },
};

use crate::v16::{
    cp::ChargePoint,
    interfaces::{ChargePointInterface, TimerId},
    state_machine::firmware::{FirmwareDownloadInfo, FirmwareState},
};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn update_firmware_ocpp(
        &mut self,
        unique_id: String,
        req: UpdateFirmwareRequest,
    ) {
        let payload = UpdateFirmwareResponse {};
        let res = CallResult::new(unique_id, payload);
        self.send_ws_msg(res.encode()).await;
        match self.firmware_state {
            FirmwareState::Idle => {
                let diff = req.retrieve_date - self.get_time().await.unwrap();
                if diff.num_seconds() > 0 {
                    self.firmware_state = FirmwareState::New(req);
                    self.add_timeout(TimerId::Firmware, diff.num_seconds() as u64)
                        .await;
                } else {
                    self.send_firmware_status_notification(FirmwareStatus::Downloading)
                        .await;
                    self.try_firmware_download(FirmwareDownloadInfo {
                        retry_left: req.retries.map(|t| t + 1).unwrap_or(1),
                        retry_interval: req.retry_interval.unwrap_or(0),
                        location: req.location,
                    })
                    .await;
                }
            }
            _ => {
                // already going on
            }
        }
    }
}
