use std::time::Duration;

use crate::harness::{event::Event, harness::CpHarness};

use super::step::{StartResult, State, StepResult};

pub struct Operation {
    next: Option<Box<dyn State>>,
    op: Option<Box<dyn FnOnce(&mut CpHarness) + Send>>,
}

impl Operation {
    pub fn new(op: impl FnOnce(&mut CpHarness) + Send + 'static) -> Self {
        Self {
            next: None,
            op: Some(Box::new(op)),
        }
    }
}

impl State for Operation {
    fn add_next(&mut self, next: Box<dyn State>) {
        self.next = Some(next)
    }

    fn handle(&mut self, ev: Event, d: Duration, h: &mut CpHarness) -> StepResult {
        if let Some(next) = self.next.as_mut() {
            next.handle(ev, d, h)
        } else {
            StepResult::Done
        }
    }

    fn on_start(&mut self, h: &mut CpHarness) -> StartResult {
        if let Some(op) = self.op.take() {
            op(h);
        }
        self.next
            .take()
            .map_or(StartResult::Done, StartResult::Next)
    }
}
