use std::time::Duration;

use anyhow::anyhow;

use crate::harness::{event::Event, harness::CpHarness};

use super::step::{StartResult, State, StepResult};

pub struct Measure {
    inner: Box<dyn State>,
    min: u64,
    max: u64,
}

impl Measure {
    pub fn new(inner: Box<dyn State>, target_ms: u64, tol_ms: u64) -> Self {
        Self {
            inner,
            min: target_ms,
            max: target_ms + tol_ms,
        }
    }

    fn ok(&self, d: Duration) -> bool {
        let el = d.as_millis() as u64;
        el >= self.min && el <= self.max
    }
}

impl State for Measure {
    fn add_next(&mut self, next: Box<dyn State>) {
        self.inner.add_next(next);
    }
    fn handle(&mut self, ev: Event, d: Duration, h: &mut CpHarness) -> StepResult {
        match self.inner.handle(ev, d, h) {
            StepResult::Next(n) => {
                if self.ok(d) {
                    StepResult::Next(n)
                } else {
                    StepResult::Fail(anyhow!(
                        "expected in [{}, {}]ms, got in {}ms",
                        self.min,
                        self.max,
                        d.as_millis()
                    ))
                }
            }
            StepResult::Done => {
                if self.ok(d) {
                    StepResult::Done
                } else {
                    StepResult::Fail(anyhow!(
                        "expected in [{}, {}]ms, got in {}ms",
                        self.min,
                        self.max,
                        d.as_millis()
                    ))
                }
            }
            other => other,
        }
    }
    fn on_start(&mut self, h: &mut CpHarness) -> StartResult {
        self.inner.on_start(h)
    }
}
