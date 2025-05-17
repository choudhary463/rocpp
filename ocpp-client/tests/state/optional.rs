use std::time::Duration;

use anyhow::anyhow;

use crate::harness::{event::Event, harness::CpHarness};

use super::step::{StartResult, State, StepResult};

pub struct Optional {
    optional: Box<dyn State>,
    started: bool,
    next: Option<Box<dyn State>>,
}

impl Optional {
    pub fn new(optional: Box<dyn State>) -> Self {
        Self {
            optional,
            started: false,
            next: None,
        }
    }
}

impl State for Optional {
    fn add_next(&mut self, next: Box<dyn State>) {
        self.next = Some(next)
    }

    fn handle(&mut self, ev: Event, d: Duration, h: &mut CpHarness) -> StepResult {
        match self.optional.handle(ev.clone(), d, h) {
            StepResult::Done => self.next.take().map_or(StepResult::Done, StepResult::Next),
            StepResult::Fail(e) => {
                if !self.started {
                    if let Some(mut t) = self.next.take() {
                        match t.handle(ev, d, h) {
                            StepResult::Pending => StepResult::Next(t),
                            StepResult::Next(state) => StepResult::Next(state),
                            StepResult::Done => StepResult::Done,
                            StepResult::Fail(e) => StepResult::Fail(e),
                        }
                    } else {
                        StepResult::Fail(e)
                    }
                } else {
                    StepResult::Fail(anyhow!("optional state not completed"))
                }
            }
            StepResult::Next(next) => {
                self.optional = next;
                self.started = true;
                StepResult::Pending
            }
            StepResult::Pending => {
                self.started = true;
                StepResult::Pending
            }
        }
    }
    fn on_start(&mut self, h: &mut CpHarness) -> StartResult {
        match self.optional.on_start(h) {
            StartResult::Stay => {
                self.started = true;
                StartResult::Stay
            }
            StartResult::Next(nxt) => {
                self.optional = nxt;
                self.started = true;
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
