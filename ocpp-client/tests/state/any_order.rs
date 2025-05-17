use std::time::Duration;

use anyhow::anyhow;

use crate::harness::{event::Event, harness::CpHarness};

use super::step::{StartResult, State, StepResult};

pub struct AnyOrder {
    todo: Vec<Box<dyn State>>,
    index: Option<usize>,
    next: Option<Box<dyn State>>,
}

impl AnyOrder {
    pub fn new(list: Vec<Box<dyn State>>) -> Self {
        Self {
            todo: list,
            index: None,
            next: None,
        }
    }
    pub fn get_index(
        &mut self,
        ev: Event,
        d: Duration,
        h: &mut CpHarness,
    ) -> Result<Option<usize>, anyhow::Error> {
        let mut errors = Vec::new();
        for index in 0..self.todo.len() {
            match self.todo[index].handle(ev.clone(), d, h) {
                StepResult::Done => {
                    self.todo.remove(index);
                    self.index = None;
                    return Ok(None);
                }
                StepResult::Pending => return Ok(Some(index)),
                StepResult::Next(nxt) => {
                    self.todo[index] = nxt;
                    return Ok(Some(index));
                }
                StepResult::Fail(e) => {
                    errors.push(e);
                }
            }
        }
        let err = errors
            .into_iter()
            .fold(anyhow!("No state in orders satisfied"), |acc, e| {
                acc.context(e)
            });
        Err(err)
    }
}

impl State for AnyOrder {
    fn add_next(&mut self, next: Box<dyn State>) {
        self.next = Some(next);
    }

    fn handle(&mut self, ev: Event, d: Duration, h: &mut CpHarness) -> StepResult {
        if self.todo.is_empty() {
            return self.next.take().map_or(StepResult::Done, StepResult::Next);
        }
        if let Some(index) = self.index {
            match self.todo[index].handle(ev, d, h) {
                StepResult::Done => {
                    self.todo.remove(index);
                    self.index = None;
                    if self.todo.is_empty() {
                        return self.next.take().map_or(StepResult::Done, StepResult::Next);
                    } else {
                        return StepResult::Pending;
                    }
                }
                StepResult::Pending => return StepResult::Pending,
                StepResult::Next(nxt) => {
                    self.todo[index] = nxt;
                    return StepResult::Pending;
                }
                StepResult::Fail(e) => return StepResult::Fail(e),
            }
        } else {
            match self.get_index(ev, d, h) {
                Ok(index) => {
                    self.index = index;
                    if self.todo.is_empty() {
                        return self.next.take().map_or(StepResult::Done, StepResult::Next);
                    } else {
                        return StepResult::Pending;
                    }
                }
                Err(e) => return StepResult::Fail(e),
            }
        }
    }
    fn on_start(&mut self, h: &mut CpHarness) -> StartResult {
        if let Some(index) = self.index {
            match self.todo[index].on_start(h) {
                StartResult::Stay => StartResult::Stay,
                StartResult::Next(nxt) => {
                    self.todo[index] = nxt;
                    StartResult::Stay
                }
                StartResult::Done => {
                    self.todo.remove(index);
                    self.index = None;
                    if self.todo.is_empty() {
                        return self
                            .next
                            .take()
                            .map_or(StartResult::Done, StartResult::Next);
                    } else {
                        return StartResult::Break;
                    }
                }
                StartResult::Break => StartResult::Break,
            }
        } else {
            StartResult::Break
        }
    }
}
