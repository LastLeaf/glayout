#![macro_use]

use std::time;

#[allow(dead_code)]
pub type Instant = time::Instant;

#[allow(unused_macros)]
macro_rules! start_measure_time {
    () => {
        $crate::utils::time::Instant::now()
    }
}

#[allow(unused_macros)]
macro_rules! end_measure_time {
    ($instant: expr) => {
        {
            let duration = $instant.elapsed();
            duration.as_secs() as i32 * 1000 + duration.subsec_nanos() as i32 / 1_000_000
        }
    }
}
