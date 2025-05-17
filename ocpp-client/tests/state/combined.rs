use std::time::Duration;

use crate::harness::{event::Event, harness::CpHarness};

use super::step::{StartResult, State, StepResult};

pub struct Combined {
    head: Box<dyn State>,
    next: Option<Box<dyn State>>,
}

impl Combined {
    pub fn new(head: Box<dyn State>) -> Self {
        Self { head, next: None }
    }
}

impl State for Combined {
    fn add_next(&mut self, next: Box<dyn State>) {
        self.next = Some(next)
    }

    fn handle(&mut self, ev: Event, d: Duration, h: &mut CpHarness) -> StepResult {
        match self.head.handle(ev.clone(), d, h) {
            StepResult::Done => self.next.take().map_or(StepResult::Done, StepResult::Next),
            StepResult::Fail(e) => StepResult::Fail(e),
            StepResult::Next(next) => {
                self.head = next;
                StepResult::Pending
            }
            StepResult::Pending => StepResult::Pending,
        }
    }
    fn on_start(&mut self, h: &mut CpHarness) -> StartResult {
        match self.head.on_start(h) {
            StartResult::Stay => StartResult::Stay,
            StartResult::Next(nxt) => {
                self.head = nxt;
                StartResult::Stay
            }
            StartResult::Done => self
                .next
                .take()
                .map_or(StartResult::Done, StartResult::Next),
            StartResult::Break => StartResult::Break,
        }
    }
}
