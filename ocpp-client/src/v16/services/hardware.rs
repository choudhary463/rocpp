use core::{future::Future, pin::Pin, task::{Context, Poll}};

use crate::v16::interface::{Hardware, HardwareActions};

pub(crate) struct HardwareService<T: Hardware> {
    hardware: T
}

impl<T: Hardware> HardwareService<T> {
    pub fn new(hardware: T) -> Self {
        Self {
            hardware
        }
    }
}

impl<T: Hardware> Future for HardwareService<T> {
    type Output = HardwareActions;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.hardware.poll_next_action(cx)
    }
}