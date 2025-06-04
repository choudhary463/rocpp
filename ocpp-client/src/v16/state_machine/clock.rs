use core::time::Duration;

use chrono::{DateTime, Utc};

use crate::v16::{
    cp::ChargePoint,
    interfaces::{ChargePointBackend, ChargePointInterface},
};

#[derive(Clone, Copy)]
pub(crate) struct Instant(u64);

impl Instant {
    pub fn default() -> Self {
        Self(0)
    }
    pub async fn now<I: ChargePointInterface>(interface: &ChargePointBackend<I>) -> Self {
        Self(interface.interface.get_boot_time().await)
    }
    pub fn duration_since(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
}

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) async fn get_time(&self) -> Option<DateTime<Utc>> {
        let now = Instant::now(&self.interface).await;
        self.base_time.map(|(base_dt, base_instant)| {
            let elapsed = now.duration_since(base_instant).0;
            base_dt + Duration::from_micros(elapsed as u64)
        })
    }
    pub(crate) fn get_time_since(&self, t: Instant) -> Option<DateTime<Utc>> {
        self.base_time.map(|(base_dt, base_instant)| {
            base_dt + Duration::from_micros(t.duration_since(base_instant).0 as u64)
        })
    }
    pub(crate) async fn set_time(&mut self, dt: DateTime<Utc>) {
        let was_uninitialized = self.base_time.is_none();
        self.base_time = Some((dt, Instant::now(&self.interface).await));
        if was_uninitialized {
            self.set_aligned_meter_sleep_state().await;
        }
    }
    pub(crate) fn default_time(&self) -> DateTime<Utc> {
        DateTime::<Utc>::MIN_UTC
    }
}
