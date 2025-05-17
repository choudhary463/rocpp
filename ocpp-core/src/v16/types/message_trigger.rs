#[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone, Debug)]
pub enum MessageTrigger {
    BootNotification,
    DiagnosticsStatusNotification,
    FirmwareStatusNotification,
    Heartbeat,
    MeterValues,
    StatusNotification,
}
