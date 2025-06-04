use core::{future::Future, pin::Pin, task::{Context, Poll}};
use std::{collections::{BTreeMap, HashMap}, time::Instant};

use rocpp_client::v16::{TimeoutScheduler, TimerId};
use tokio::time::Sleep;

pub struct TokioTimerServie {
    timer_deadlines: HashMap<TimerId, Instant>,
    deadline_queue: BTreeMap<Instant, TimerId>,
    active_sleep: Option<(Pin<Box<Sleep>>, TimerId)>,
    needs_reschedule: bool,
}

impl TokioTimerServie {
    pub fn new() -> Self {
        Self {
            timer_deadlines: HashMap::new(),
            deadline_queue: BTreeMap::new(),
            active_sleep: None,
            needs_reschedule: false,
        }
    }
    fn next_deadline(&self) -> Option<(Instant, TimerId)> {
        self.deadline_queue.iter().next().map(|(k, v)| (*k, *v))
    }
}

impl TimeoutScheduler for TokioTimerServie {
    async fn add_or_update_timeout(&mut self, id: TimerId, timeout: u64) {
        let when = Instant::now() + std::time::Duration::from_secs(timeout);
        if let Some(prev) = self.timer_deadlines.insert(id, when) {
            self.deadline_queue.remove(&prev);
        }
        self.deadline_queue.insert(when, id);
        self.needs_reschedule = true;
    }

    async fn remove_timeout(&mut self, id: TimerId) {
        if let Some(inst) = self.timer_deadlines.remove(&id) {
            self.deadline_queue.remove(&inst);
        }
        self.needs_reschedule = true;
    }

    async fn remove_all_timeouts(&mut self) {
        self.timer_deadlines.clear();
        self.deadline_queue.clear();
        self.active_sleep = None;
        self.needs_reschedule = false;
    }

    fn poll_timeout(&mut self, cx: &mut Context<'_>) -> Poll<TimerId> {
        let expired_timer_id = if self.needs_reschedule {
            let (deadline, timer_id) = match self.next_deadline() {
                Some(v) => v,
                None => return Poll::Pending,
            };
            let now = Instant::now();
            if now >= deadline {
                timer_id
            } else {
                let sleep_duration = deadline - now;
                self.needs_reschedule = false;
                self.active_sleep = Some((Box::pin(tokio::time::sleep(sleep_duration)), timer_id));
                match &mut self.active_sleep {
                    Some((sleep_fut, id)) => match sleep_fut.as_mut().poll(cx) {
                        Poll::Ready(()) => *id,
                        Poll::Pending => return Poll::Pending,
                    },
                    None => return Poll::Pending,
                }
            }
        } else {
            match &mut self.active_sleep {
                Some((sleep_fut, id)) => match sleep_fut.as_mut().poll(cx) {
                    Poll::Ready(()) => *id,
                    Poll::Pending => return Poll::Pending,
                },
                None => return Poll::Pending,
            }
        };
        self.needs_reschedule = true;
        Poll::Ready(expired_timer_id)
    }
}