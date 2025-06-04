use std::{fmt::Debug, time::Duration};

use anyhow::anyhow;
use rocpp_core::v16::protocol_error::ProtocolError;

use crate::harness::{
    event::{ConnectionEvents, Event},
    harness::CpHarness,
};

use super::step::{State, StepResult, TestChain};

pub enum AfterValidation {
    Failed(anyhow::Error),
    NextDefault,
    NextCustom(Box<dyn State>),
}

pub struct AwaitWsMsg<T> {
    validator: Option<Box<dyn FnOnce(&Result<T, ProtocolError>) -> AfterValidation + Send>>,
    next: Option<Box<dyn State>>,
}

impl<T> AwaitWsMsg<T> {
    pub fn new() -> Self {
        Self {
            validator: None,
            next: None,
        }
    }
    fn set_validator(
        &mut self,
        v: impl FnOnce(&Result<T, ProtocolError>) -> AfterValidation + Send + 'static,
    ) {
        self.validator = Some(Box::new(v));
    }
}

impl<T: serde::de::DeserializeOwned> State for AwaitWsMsg<T> {
    fn add_next(&mut self, next: Box<dyn State>) {
        self.next = Some(next);
    }
    fn handle(&mut self, ev: Event, _duration: Duration, _h: &mut CpHarness) -> StepResult {
        if let Event::Connection(ConnectionEvents::WsMsg(call, msg)) = ev {
            if let Some(call) = call {
                let action = std::any::type_name::<T>()
                    .rsplit("::")
                    .next()
                    .and_then(|t| t.strip_suffix("Request"))
                    .unwrap()
                    .to_string();
                if action != call {
                    return StepResult::Fail(anyhow!("expected {} action, found {}", action, call));
                }
            }
            let msg = match msg {
                Ok(v) => match serde_json::from_value::<T>(v) {
                    Ok(t) => Ok(t),
                    Err(e) => return StepResult::Fail(e.into()),
                },
                Err(e) => Err(e),
            };
            let validator = self.validator.take().unwrap();
            match validator(&msg) {
                AfterValidation::Failed(e) => StepResult::Fail(e),
                AfterValidation::NextDefault => {
                    self.next.take().map_or(StepResult::Done, StepResult::Next)
                }
                AfterValidation::NextCustom(next) => StepResult::Next(next),
            }
        } else {
            return StepResult::Fail(anyhow!(
                "expected WS msg {}",
                std::any::type_name::<T>().rsplit("::").next().unwrap()
            ));
        }
    }
}

pub struct AwaitWsMsgBuilder<T> {
    current: TestChain,
    res: AwaitWsMsg<T>,
    default_validations: Vec<Box<dyn FnOnce(&T) -> Option<String> + Send>>,
}

impl<T: serde::de::DeserializeOwned + 'static> AwaitWsMsgBuilder<T> {
    pub fn check<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> Option<String> + Send + 'static,
    {
        self.default_validations
            .push(Box::new( move |t: &T| f(t) ));
        self
    }
    pub fn check_eq<F, U>(mut self, expected: &U, f: F) -> Self
    where
        F: Fn(&T) -> &U + Send + 'static,
        U: PartialEq + Send + Clone + Debug + 'static,
    {
        self.default_validations.push(Box::new({
            let expected = expected.clone();
            move |t: &T| {
                let found = f(t);
                (f(t) != &expected).then_some(format!("expected {:?}, found {:?}", expected, found))
            }
        }));
        self
    }
    pub fn done(mut self) -> TestChain {
        let validator = move |t: &Result<T, ProtocolError>| match t {
            Ok(t) => {
                for f in self.default_validations {
                    if let Some(e) = f(t) {
                        return AfterValidation::Failed(anyhow!(e));
                    }
                }
                AfterValidation::NextDefault
            }
            Err(e) => {
                AfterValidation::Failed(anyhow!("expected payload, found ProtocolError {:?}", e))
            }
        };
        self.res.set_validator(validator);
        self.current.next(self.res)
    }
    pub fn done_custom(
        mut self,
        v: impl FnOnce(&Result<T, ProtocolError>) -> AfterValidation + Send + 'static,
    ) -> TestChain {
        self.res.set_validator(v);
        self.current.next(self.res)
    }
}

impl TestChain {
    pub fn await_ws_msg<T: serde::de::DeserializeOwned + 'static>(self) -> AwaitWsMsgBuilder<T> {
        AwaitWsMsgBuilder {
            current: self,
            res: AwaitWsMsg::new(),
            default_validations: Vec::new(),
        }
    }
}
