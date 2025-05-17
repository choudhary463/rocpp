use std::time::Instant;

use chrono::{DateTime, Utc};

use crate::v16::interface::{Database, Secc};

use super::core::ChargePointCore;

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub fn get_time(&self) -> Option<DateTime<Utc>> {
        self.base_time
            .map(|(base_dt, base_instant)| base_dt + Instant::now().duration_since(base_instant))
    }
    pub fn get_time_since(&self, t: Instant) -> Option<DateTime<Utc>> {
        self.base_time
            .map(|(base_dt, base_instant)| base_dt + t.duration_since(base_instant))
    }
    pub fn set_time(&mut self, dt: DateTime<Utc>) {
        let was_uninitialized = self.base_time.is_none();
        self.base_time = Some((dt, Instant::now()));
        if was_uninitialized {
            self.set_aligned_meter_sleep_state();
        }
    }
    pub fn default_time(&self) -> DateTime<Utc> {
        DateTime::<Utc>::MIN_UTC
    }
}
