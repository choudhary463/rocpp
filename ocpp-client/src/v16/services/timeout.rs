use core::{future::Future, pin::Pin, task::{Context, Poll}};

use crate::v16::{interface::Timeout, TimerId};

pub(crate) struct TimeoutService<T: Timeout> {
    timeout: T
}

impl<T: Timeout> TimeoutService<T> {
    pub fn new(timeout: T) -> Self {
        Self {
            timeout
        }
    }

    pub fn add_or_update(&mut self, id: TimerId, timeout: u64) {
        self.timeout.add_or_update_timeout(id, timeout);
    }

    pub fn remove_timeout(&mut self, id: TimerId) {
        self.timeout.remove_timeout(id);
    }

    pub fn remove_all_timeouts(&mut self) {
        self.timeout.remove_all_timeouts();
    }
}

impl<T: Timeout> Future for TimeoutService<T> {
    type Output = TimerId;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.timeout.poll_timeout(cx)
    }
}


