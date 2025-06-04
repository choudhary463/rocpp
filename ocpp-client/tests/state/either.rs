use std::{mem::swap, time::Duration};

use anyhow::anyhow;

use crate::harness::{event::Event, harness::CpHarness};

use super::step::{StartResult, State, StepResult};

pub struct Either {
    a: Box<dyn State>,
    b: Box<dyn State>,
    started: bool,
    next: Option<Box<dyn State>>,
}

impl Either {
    pub fn new(a: Box<dyn State>, b: Box<dyn State>) -> Self {
        Self {
            a,
            b,
            started: false,
            next: None,
        }
    }
}

impl State for Either {
    fn add_next(&mut self, next: Box<dyn State>) {
        self.next = Some(next)
    }

    fn handle(&mut self, ev: Event, d: Duration, h: &mut CpHarness) -> StepResult {
        if self.started {
            match self.a.handle(ev, d, h) {
                StepResult::Pending => StepResult::Pending,
                StepResult::Next(next) => {
                    self.a = next;
                    StepResult::Pending
                }
                StepResult::Done => self.next.take().map_or(StepResult::Done, StepResult::Next),
                StepResult::Fail(e) => StepResult::Fail(e),
            }
        } else {
            let err1 = match self.a.handle(ev.clone(), d, h) {
                StepResult::Pending => {
                    self.started = true;
                    None
                }
                StepResult::Next(next) => {
                    self.started = true;
                    self.a = next;
                    None
                }
                StepResult::Done => {
                    return self.next.take().map_or(StepResult::Done, StepResult::Next);
                }
                StepResult::Fail(e) => Some(e),
            };
            if !self.started {
                let err2 = match self.b.handle(ev, d, h) {
                    StepResult::Pending => {
                        self.started = true;
                        None
                    }
                    StepResult::Next(next) => {
                        self.started = true;
                        self.b = next;
                        None
                    }
                    StepResult::Done => {
                        return self.next.take().map_or(StepResult::Done, StepResult::Next);
                    }
                    StepResult::Fail(e) => Some(e),
                };
                if self.started {
                    swap(&mut self.a, &mut self.b);
                    StepResult::Pending
                } else {
                    StepResult::Fail(anyhow!("either failed {} {}", err1.unwrap(), err2.unwrap()))
                }
            } else {
                StepResult::Pending
            }
        }
    }
    fn on_start(&mut self, h: &mut CpHarness) -> StartResult {
        if !self.started {
            return StartResult::Break;
        }
        self.a.on_start(h)
    }
}
