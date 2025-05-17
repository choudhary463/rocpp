use std::time::Duration;

use anyhow::anyhow;

use crate::harness::{
    event::{ConnectionEvents, Event},
    harness::CpHarness,
};

use super::step::{State, StepResult, TestChain};

pub struct AwaitConnection {
    url: String,
    with_disconnection: bool,
    next: Option<Box<dyn State>>,
}

impl AwaitConnection {
    pub fn new(url: String, with_disconnection: bool) -> Self {
        Self {
            url,
            with_disconnection,
            next: None,
        }
    }
}

impl State for AwaitConnection {
    fn add_next(&mut self, next: Box<dyn State>) {
        self.next = Some(next);
    }
    fn handle(&mut self, ev: Event, _duration: Duration, _h: &mut CpHarness) -> StepResult {
        if !self.with_disconnection {
            match ev {
                Event::Connection(ConnectionEvents::Connected(t)) => {
                    if t == self.url {
                        if let Some(next) = self.next.take() {
                            StepResult::Next(next)
                        } else {
                            StepResult::Done
                        }
                    } else {
                        StepResult::Fail(anyhow!(
                            "expected connection from `{}` but got `{}`",
                            self.url,
                            t
                        ))
                    }
                }
                _ => StepResult::Fail(anyhow!("expected Connection::Connected Event")),
            }
        } else {
            match ev {
                Event::Connection(ConnectionEvents::Disconnected) => {
                    self.with_disconnection = false;
                    StepResult::Pending
                }
                _ => StepResult::Fail(anyhow!("expected Connection::Disconnected Event")),
            }
        }
    }
}

struct AwaitTimeout {
    next: Option<Box<dyn State>>,
}

impl AwaitTimeout {
    pub fn new() -> Self {
        Self { next: None }
    }
}

impl State for AwaitTimeout {
    fn handle(&mut self, ev: Event, _duration: Duration, _h: &mut CpHarness) -> StepResult {
        match ev {
            Event::Connection(ConnectionEvents::Timeout) => {
                self.next.take().map_or(StepResult::Done, StepResult::Next)
            }
            _ => StepResult::Fail(anyhow!("expected timeout")),
        }
    }

    fn add_next(&mut self, next: Box<dyn State>) {
        self.next = Some(next);
    }
}

struct AwaitDisconnection {
    next: Option<Box<dyn State>>,
}

impl AwaitDisconnection {
    pub fn new() -> Self {
        Self { next: None }
    }
}

impl State for AwaitDisconnection {
    fn handle(&mut self, ev: Event, _duration: Duration, _h: &mut CpHarness) -> StepResult {
        match ev {
            Event::Connection(ConnectionEvents::Disconnected) => {
                self.next.take().map_or(StepResult::Done, StepResult::Next)
            }
            _ => StepResult::Fail(anyhow!("expected Disconnect")),
        }
    }

    fn add_next(&mut self, next: Box<dyn State>) {
        self.next = Some(next);
    }
}

impl TestChain {
    pub fn await_connection(self, url: String, with_disconnection: bool) -> Self {
        self.next(AwaitConnection::new(url, with_disconnection))
    }
    pub fn await_timeout(self) -> Self {
        self.next(AwaitTimeout::new())
    }
    pub fn await_disconnection(self) -> Self {
        self.next(AwaitDisconnection::new())
    }
}
