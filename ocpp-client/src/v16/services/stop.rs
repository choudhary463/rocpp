use core::{future::Future, pin::Pin, task::{Context, Poll}};

use crate::v16::interface::StopChargePoint;

pub(crate) struct StopChargePointService<T: StopChargePoint> {
    stop: T
}

impl<T: StopChargePoint> StopChargePointService<T> {
    pub fn new(stop: T) -> Self {
        Self {
            stop
        }
    }
}

impl<T: StopChargePoint> Future for StopChargePointService<T> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.stop.poll_stopped(cx)
    }
}