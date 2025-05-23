use core::time::Duration;

use chrono::{DateTime, Utc};

use crate::v16::{interface::{Database, Secc}, services::secc::SeccService, cp::ChargePointCore};

#[derive(Clone, Copy)]
pub struct Instant(u128);

impl Instant {
    pub fn default() -> Self {
        Self(0)
    }
    pub fn now<S: Secc>(secc: &SeccService<S>) -> Self {
        Self(secc.get_boot_time())
    }
    pub fn duration_since(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
}

impl<D: Database, S: Secc> ChargePointCore<D, S> {
    pub(crate) fn get_time(&self) -> Option<DateTime<Utc>> {
        self.base_time
            .map(|(base_dt, base_instant)| base_dt + Duration::from_micros(Instant::now(&self.secc).duration_since(base_instant).0 as u64))
    }
    pub(crate) fn get_time_since(&self, t: Instant) -> Option<DateTime<Utc>> {
        self.base_time
            .map(|(base_dt, base_instant)| base_dt + Duration::from_micros(t.duration_since(base_instant).0 as u64))
    }
    pub(crate) fn set_time(&mut self, dt: DateTime<Utc>) {
        let was_uninitialized = self.base_time.is_none();
        self.base_time = Some((dt, Instant::now(&self.secc)));
        if was_uninitialized {
            self.set_aligned_meter_sleep_state();
        }
    }
    pub(crate) fn default_time(&self) -> DateTime<Utc> {
        DateTime::<Utc>::MIN_UTC
    }
}
