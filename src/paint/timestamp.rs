use lazy_static::lazy_static;

use std::time::Instant;

lazy_static! {
    static ref BOOT_INSTANT: Instant = Instant::now();
}

pub fn now() -> u64 {
    BOOT_INSTANT.elapsed().as_millis() as u64
}
