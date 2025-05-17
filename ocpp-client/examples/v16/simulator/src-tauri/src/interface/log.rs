use std::sync::OnceLock;

use log::{Level, LevelFilter, Metadata, Record};

use super::ui::UiClient;

pub struct UiLogger;

static LOGGER: UiLogger = UiLogger;
static UI_CLIENT: OnceLock<UiClient> = OnceLock::new();

pub fn init_log(ui: UiClient, level: LevelFilter) {
    UI_CLIENT.set(ui).unwrap();
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(level);
}

impl log::Log for UiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if let Some(path) = record.module_path() {
            if path.starts_with("simulator") || path.starts_with("ocpp_client") {
                if let Some(t) = UI_CLIENT.get() {
                    let kind = match record.level() {
                        Level::Error => "ERROR",
                        Level::Warn => "WARN",
                        Level::Info => "INFO",
                        Level::Debug => "DEBUG",
                        Level::Trace => "TRACE",
                    };
                    let msg = format!("{}", record.args());
                    t.log(kind.to_string(), msg);
                }
            }
        }
    }

    fn flush(&self) {}
}
