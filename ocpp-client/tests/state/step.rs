use rocpp_client::v16::{HardwareEvent, SeccState};
use serde::Serialize;
use std::{path::PathBuf, time::Duration};

use crate::{
    harness::{
        event::{ConnectionEvents, Event},
        harness::CpHarness,
    },
    state::combined::Combined,
};

use super::{
    any_order::AnyOrder, either::Either, measure::Measure, operation::Operation, optional::Optional, timestamp::WithNowTimestamp
};

pub enum StepResult {
    Pending,
    Next(Box<dyn State>),
    Done,
    Fail(anyhow::Error),
}

pub enum StartResult {
    Stay,
    Next(Box<dyn State>),
    Done,
    Break,
}

pub trait State: Send {
    fn handle(&mut self, ev: Event, d: Duration, h: &mut CpHarness) -> StepResult;
    fn add_next(&mut self, next: Box<dyn State>);
    fn on_start(&mut self, _h: &mut CpHarness) -> StartResult {
        StartResult::Break
    }
}

// #[derive(Debug)]
pub enum TestKind {
    State(Box<dyn State>),
    WithTiming(u64, u64),
    MergeToAny(usize),
    Optional(usize),
    Operation(Box<dyn FnOnce(&mut CpHarness) + Send>),
    Combine(usize),
    Either
}

impl std::fmt::Debug for TestKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestKind::State(_) => f.write_str("State(<dyn State>)"),
            TestKind::WithTiming(a, b) => f.debug_tuple("WithTiming").field(a).field(b).finish(),
            TestKind::MergeToAny(i) => f.debug_tuple("MergeToAny").field(i).finish(),
            TestKind::Optional(i) => f.debug_tuple("Optional").field(i).finish(),
            TestKind::Operation(_) => f.write_str("Operation(<FnOnce>)"),
            TestKind::Combine(i) => f.debug_tuple("Combine").field(i).finish(),
            TestKind::Either => f.debug_tuple("Either").finish(),
        }
    }
}

#[derive(Debug)]
pub struct TestChain {
    pub list: Vec<TestKind>,
}

impl TestChain {
    pub fn new() -> Self {
        Self { list: vec![] }
    }

    pub fn next(mut self, s: impl State + 'static) -> Self {
        self.list.push(TestKind::State(Box::new(s)));
        self
    }

    pub fn merge(mut self, mut other: Self) -> Self {
        self.list.append(&mut other.list);
        self
    }

    pub fn merge_into_one(mut self, other: Self) -> Self {
        let state = Self::build(other);
        self.list.push(TestKind::State(state));
        self
    }

    pub fn operation(mut self, cb: impl FnOnce(&mut CpHarness) + Send + 'static) -> Self {
        self.list.push(TestKind::Operation(Box::new(cb)));
        self
    }

    pub fn any_order(mut self, num: usize) -> Self {
        self.list.push(TestKind::MergeToAny(num));
        self
    }

    pub fn optional(mut self, num: usize) -> Self {
        self.list.push(TestKind::Optional(num));
        self
    }

    pub fn with_timing(mut self, t: u64, tol: u64) -> Self {
        self.list.push(TestKind::WithTiming(t, tol));
        self
    }

    pub fn combine(mut self, num: usize) -> Self {
        self.list.push(TestKind::Combine(num));
        self
    }

    pub fn either(mut self) -> Self {
        self.list.push(TestKind::Either);
        self
    }

    pub fn pop(mut self) -> Self {
        self.list.pop();
        self
    }

    fn chain(v: Vec<Box<dyn State>>) -> Box<dyn State> {
        let mut iter = v.into_iter().rev();
        let mut chain = iter.next().unwrap();
        for mut s in iter {
            s.add_next(chain);
            chain = s;
        }
        chain
    }

    pub fn build(self) -> Box<dyn State> {
        let mut res = Vec::new();
        for kind in self.list {
            match kind {
                TestKind::State(t) => res.push(t),
                TestKind::WithTiming(t, tol) => {
                    if let Some(last) = res.pop() {
                        res.push(Box::new(Measure::new(last, t, tol)))
                    } else {
                        panic!("WithTiming called with no states in chain");
                    }
                }
                TestKind::MergeToAny(num) => {
                    let len = res.len().saturating_sub(num);
                    let last_list = res.split_off(len);
                    res.push(Box::new(AnyOrder::new(last_list)));
                }
                TestKind::Operation(t) => {
                    res.push(Box::new(Operation::new(t)));
                }
                TestKind::Optional(num) => {
                    let len = res.len().saturating_sub(num);
                    let last_list = res.split_off(len);
                    res.push(Box::new(Optional::new(Self::chain(last_list))));
                }
                TestKind::Combine(num) => {
                    let len = res.len().saturating_sub(num);
                    let last_list = res.split_off(len);
                    res.push(Box::new(Combined::new(Self::chain(last_list))));
                }
                TestKind::Either => {
                    let b = res.pop().unwrap();
                    let a = res.pop().unwrap();
                    res.push(Box::new(Either::new(a, b)));
                }
            }
        }
        Self::chain(res)
    }

    pub async fn run(
        self,
        timeout: u64,
        override_defualt_configs: Vec<(&str, &str)>,
        db_dir: Option<PathBuf>,
    ) {
        let mut h = CpHarness::new(timeout, override_defualt_configs, db_dir, true);
        let mut st = self.build();
        loop {
            loop {
                tokio::task::yield_now().await;
                match st.on_start(&mut h) {
                    StartResult::Done => {
                        h.stop_token.cancel();
                        return;
                    }
                    StartResult::Next(next) => {
                        st = next;
                    }
                    StartResult::Break => {
                        break;
                    }
                    StartResult::Stay => {}
                }

            }
            let (ev, d) = h.bus_rx.next().await.unwrap_or((
                Event::Connection(ConnectionEvents::Timeout),
                Duration::from_secs(timeout),
            ));
            log::info!("received test event: {:?}", ev);
            match st.handle(ev, d, &mut h) {
                StepResult::Pending => {}
                StepResult::Next(nxt) => {
                    st = nxt;
                }
                StepResult::Done => {
                    h.stop_token.cancel();
                    return
                },
                StepResult::Fail(msg) => {
                    h.stop_token.cancel();
                    panic!("{:?}", msg);
                },
            }
            tokio::task::yield_now().await;
        }
    }
}

impl TestChain {
    pub fn call<T: Serialize + Send + 'static>(self, payload: T) -> Self {
        let action = std::any::type_name::<T>()
            .rsplit("::")
            .next()
            .and_then(|t| t.strip_suffix("Request"))
            .unwrap();
        self.operation(|t| {
            t.ws_handle.send_call(action, payload);
        })
    }
    pub fn respond<T: Serialize + Send + 'static>(mut self, payload: T) -> Self {
        self = self.operation(|t| {
            t.ws_handle.send_response(Ok(payload));
        });
        self.combine(2)
    }
    pub fn respond_with_now<T: Serialize + Send + WithNowTimestamp + 'static>(
        mut self,
        payload: T,
    ) -> Self {
        let payload = payload.with_now();
        self = self.operation(|t| {
            t.ws_handle.send_response(Ok(payload));
        });
        self.combine(2)
    }
    pub fn plug(self, connector_id: usize) -> Self {
        self.operation(move |t| {
            t.hardware_tx
                .send(HardwareEvent::State(
                    connector_id - 1,
                    SeccState::Plugged,
                    None,
                    None,
                ))
                .unwrap();
        })
    }
    pub fn unplug(self, connector_id: usize) -> Self {
        self.operation(move |t| {
            t.hardware_tx
                .send(HardwareEvent::State(
                    connector_id - 1,
                    SeccState::Unplugged,
                    None,
                    None,
                ))
                .unwrap();
        })
    }
    pub fn faulty(self, connector_id: usize) -> Self {
        self.operation(move |t| {
            t.hardware_tx
                .send(HardwareEvent::State(
                    connector_id - 1,
                    SeccState::Faulty,
                    None,
                    None,
                ))
                .unwrap();
        })
    }
    pub fn present_id_tag(self, connector_id: usize, id_tag: String) -> Self {
        self.operation(move |t| {
            t.hardware_tx
                .send(HardwareEvent::IdTag(connector_id - 1, id_tag))
                .unwrap();
        })
    }
    pub fn spawn_new(
        self,
        timeout: u64,
        override_defualt_configs: Vec<(&'static str, &'static str)>,
        db_dir: Option<PathBuf>,
        clear_db: bool,
    ) -> Self {
        self.operation(move |t| {
            *t = CpHarness::new(timeout, override_defualt_configs, db_dir, clear_db)
        })
    }
    pub fn cut_power(self) -> Self {
        self.operation(|t| {
            t.stop_token.cancel();
        })
    }
    pub fn close_connection(self) -> Self {
        self.operation(|t| {
            t.ws_handle.close_connection();
        })
    }
    pub fn restore_connection(self) -> Self {
        self.operation(|t| {
            t.ws_handle.restore_connection();
        })
    }
}

#[macro_export]
macro_rules! test_chain {
    // await_ws_msg(Type { ... })
    ($start:expr,
        await_ws_msg($t:ty { $($field:ident : $val:expr),* $(,)? }) $(, $($rest:tt)*)? ) => {
           test_chain!(
               {
                   #[allow(unused_mut)]
                   let mut b = $start.await_ws_msg::<$t>();
                   $(
                       b = b.check_eq(&$val, |m| &m.$field);
                   )*
                   b.done()
               }
               $(, $($rest)*)?
           )
       };

    // call(payload)
    ($start:expr,
     call($payload:expr) $(, $($rest:tt)*)? ) => {
        test_chain!($start.call($payload) $(, $($rest)*)? )
    };

    // respond(payload)
    ($start:expr,
     respond($payload:expr) $(, $($rest:tt)*)? ) => {
        test_chain!($start.respond($payload) $(, $($rest)*)? )
    };

    // respond_with_now(payload)
    ($start:expr,
     respond_with_now($payload:expr) $(, $($rest:tt)*)? ) => {
        test_chain!($start.respond_with_now($payload) $(, $($rest)*)? )
    };

    // plug(id)
    ($start:expr,
     plug($id:expr) $(, $($rest:tt)*)? ) => {
        test_chain!($start.plug($id) $(, $($rest)*)? )
    };

    // unplug(id)
    ($start:expr,
     unplug($id:expr) $(, $($rest:tt)*)? ) => {
        test_chain!($start.unplug($id) $(, $($rest)*)? )
    };

    // faulty(id)
    ($start:expr,
    faulty($id:expr) $(, $($rest:tt)*)? ) => {
        test_chain!($start.faulty($id) $(, $($rest)*)? )
    };

    // present_id_tag(id, tag)
    ($start:expr,
     present_id_tag($id:expr, $tag:expr) $(, $($rest:tt)*)? ) => {
        test_chain!($start.present_id_tag($id, $tag) $(, $($rest)*)? )
    };

    // spawn_new(to,ov,persist,clear)
    ($start:expr,
     spawn_new($timeout:expr, $ov:expr, $pers:expr, $clear:expr) $(, $($rest:tt)*)? ) => {
        test_chain!(
            $start.spawn_new($timeout, $ov, $pers, $clear)
            $(, $($rest)*)?
        )
    };
    // merge(num)
    ($start:expr, merge($n:expr) $(, $($rest:tt)*)? ) => {
        test_chain!($start.merge($n) $(, $($rest)*)? )
    };

    // merger_into_one(num)
    ($start:expr, merge_into_one($n:expr) $(, $($rest:tt)*)? ) => {
        test_chain!($start.merge_into_one($n) $(, $($rest)*)? )
    };

    // any_order(num)
    ($start:expr, any_order($n:expr) $(, $($rest:tt)*)? ) => {
        test_chain!($start.any_order($n) $(, $($rest)*)? )
    };

    // optional(num)
    ($start:expr, optional($n:expr) $(, $($rest:tt)*)? ) => {
        test_chain!($start.optional($n) $(, $($rest)*)? )
    };

    // either()
    ($start:expr, either() $(, $($rest:tt)*)? ) => {
        test_chain!($start.either() $(, $($rest)*)? )
    };

    // with_timing(tol, t)
    ($start:expr, with_timing($t:expr, $tol:expr) $(, $($rest:tt)*)? ) => {
        test_chain!($start.with_timing($t, $tol) $(, $($rest)*)? )
    };

    // combine(num)
    ($start:expr, combine($n:expr) $(, $($rest:tt)*)? ) => {
        test_chain!($start.combine($n) $(, $($rest)*)? )
    };

    // pop()
    ($start:expr, pop() $(, $($rest:tt)*)? ) => {
        test_chain!($start.pop() $(, $($rest)*)? )
    };

    // await_connection(with_disconnection)
    ($start:expr, await_connection($url:expr, $with_disconnection:expr) $(, $($rest:tt)*)? ) => {
        test_chain!(
            $start.await_connection($url, $with_disconnection)
            $(, $($rest)*)?
        )
    };

    // await_timeout()
    ($start:expr, await_timeout() $(, $($rest:tt)*)? ) => {
        test_chain!(
            $start.await_timeout()
            $(, $($rest)*)?
        )
    };

    // await_hard_reset()
    ($start:expr, await_hard_reset() $(, $($rest:tt)*)? ) => {
        test_chain!(
            $start.await_hard_reset()
            $(, $($rest)*)?
        )
    };

    // cut_power()
    ($start:expr, cut_power() $(, $($rest:tt)*)? ) => {
        test_chain!(
            $start.cut_power()
            $(, $($rest)*)?
        )
    };

    // close_connection()
    ($start:expr, close_connection() $(, $($rest:tt)*)? ) => {
        test_chain!(
            $start.close_connection()
            $(, $($rest)*)?
        )
    };

    // restore_connection()
    ($start:expr, restore_connection() $(, $($rest:tt)*)? ) => {
        test_chain!(
            $start.restore_connection()
            $(, $($rest)*)?
        )
    };

    // await_disconnection()
    ($start:expr, await_disconnection() $(, $($rest:tt)*)? ) => {
        test_chain!(
            $start.await_disconnection()
            $(, $($rest)*)?
        )
    };

    // base case
    ($start:expr $(,)?) => {
        $start
    };
}
