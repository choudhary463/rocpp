#[cfg(feature = "async")]
use core::{future::Future, pin::Pin, task::{Context, Poll}};

#[cfg(feature = "tokio_timer")]
use {std::collections::{HashMap, BTreeMap}, std::time::{Duration, Instant}, tokio::time::Sleep};

#[derive(Eq, Hash, Clone, Copy, PartialEq, Debug)]
pub enum TimerId {
    Boot,
    Heartbeat,
    Call,
    StatusNotification(usize),
    Transaction,
    Authorize(usize),
    Reservation(usize),
    Firmware,
    MeterAligned,
    MeterSampled(usize),
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait TimerManager: Send + Unpin + 'static {
    fn add_or_update_timeout(&mut self, id: TimerId, timeout: u64);
    fn remove_timeout(&mut self, id: TimerId);
    fn remove_all_timeouts(&mut self);
    fn poll_timeout(&mut self, cx: &mut Context<'_>) -> Poll<TimerId>;
}

#[cfg(feature = "async")]
pub(crate) struct TimerDriver<T: TimerManager> {
    timer: T
}

#[cfg(feature = "async")]
impl<T: TimerManager> TimerDriver<T> {
    pub fn new(timer: T) -> Self {
        Self {
            timer
        }
    }

    pub fn add_or_update(&mut self, id: TimerId, timeout: u64) {
        self.timer.add_or_update_timeout(id, timeout);
    }

    pub fn remove_timeout(&mut self, id: TimerId) {
        self.timer.remove_timeout(id);
    }

    pub fn remove_all_timeouts(&mut self) {
        self.timer.remove_all_timeouts();
    }
}

#[cfg(feature = "async")]
impl<T: TimerManager> Future for TimerDriver<T> {
    type Output = TimerId;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.timer.poll_timeout(cx)
    }
}

#[cfg(feature = "tokio_timer")]
pub struct TokioTimerManager {
    timer_deadlines: HashMap<TimerId, Instant>,
    deadline_queue: BTreeMap<Instant, TimerId>,
    active_sleep: Option<(Pin<Box<Sleep>>, TimerId)>,
    needs_reschedule: bool,
}

#[cfg(feature = "tokio_timer")]
impl TokioTimerManager {
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

#[cfg(feature = "tokio_timer")]
impl TimerManager for TokioTimerManager {
    fn add_or_update_timeout(&mut self, id: TimerId, timeout: u64) {
        let when = Instant::now() + Duration::from_secs(timeout);
        if let Some(prev) = self.timer_deadlines.insert(id, when) {
            self.deadline_queue.remove(&prev);
        }
        self.deadline_queue.insert(when, id);
        self.needs_reschedule = true;
    }

    fn remove_timeout(&mut self,id:TimerId) {
        if let Some(inst) = self.timer_deadlines.remove(&id) {
            self.deadline_queue.remove(&inst);
        }
        self.needs_reschedule = true;
    }

    fn remove_all_timeouts(&mut self) {
        self.timer_deadlines.clear();
        self.deadline_queue.clear();
        self.active_sleep = None;
        self.needs_reschedule = false;
    }

    fn poll_timeout(&mut self, cx: &mut Context<'_>) -> Poll<TimerId>  {
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