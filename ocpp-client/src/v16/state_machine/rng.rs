use alloc::string::String;
use core::fmt::Write;
use rand::TryRngCore;

use crate::v16::{cp::ChargePoint, interfaces::ChargePointInterface};

impl<I: ChargePointInterface> ChargePoint<I> {
    pub(crate) fn get_uuid(&mut self) -> String {
        let mut bytes = [0u8; 16];
        self.rng.try_fill_bytes(&mut bytes).unwrap();
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        let mut s = String::with_capacity(36);
        for (i, b) in bytes.iter().enumerate() {
            if i == 4 || i == 6 || i == 8 || i == 10 {
                s.push('-');
            }
            write!(s, "{:02x}", b).unwrap();
        }
        s
    }
}
