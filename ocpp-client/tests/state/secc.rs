use std::time::Duration;

use anyhow::anyhow;

use crate::harness::{
    event::{Event, SeccEvents},
    harness::CpHarness,
};

use super::step::{State, StepResult, TestChain};

struct AwaitHardReboot {
    next: Option<Box<dyn State>>,
}

impl AwaitHardReboot {
    pub fn new() -> Self {
        Self { next: None }
    }
}

impl State for AwaitHardReboot {
    fn handle(&mut self, ev: Event, _duration: Duration, _h: &mut CpHarness) -> StepResult {
        match ev {
            Event::Secc(SeccEvents::HardReset) => {
                self.next.take().map_or(StepResult::Done, StepResult::Next)
            }
            _ => StepResult::Fail(anyhow!("expected HardReset")),
        }
    }

    fn add_next(&mut self, next: Box<dyn State>) {
        self.next = Some(next);
    }
}

impl TestChain {
    pub fn await_hard_reset(self) -> Self {
        self.next(AwaitHardReboot::new())
    }
}
