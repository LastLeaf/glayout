#![macro_use]

use std::sync::{Arc, RwLock};

lazy_static! {
    static ref LOG_LEVEL_NUM: Arc<RwLock<Box<i32>>> = Arc::new(RwLock::new(Box::new(0)));
}

#[allow(dead_code)]
pub fn set_log_level_num(num: i32) {
    **LOG_LEVEL_NUM.write().unwrap() = num;
}

#[inline]
pub fn log_level_num() -> i32 {
    **LOG_LEVEL_NUM.read().unwrap()
}

#[allow(unused_macros)]
macro_rules! debug {
    ($($c:tt)*) => {
        if $crate::utils::log_level::log_level_num() <= -1 {
            print!("[glayout] [debug] ");
            println!($($c)*);
        }
    }
}

#[allow(unused_macros)]
macro_rules! log {
    ($($c:tt)*) => {
        if $crate::utils::log_level::log_level_num() <= 0 {
            print!("[glayout] [log] ");
            println!($($c)*);
        }
    }
}

#[allow(unused_macros)]
macro_rules! info {
    ($($c:tt)*) => {
        if $crate::utils::log_level::log_level_num() <= 1 {
            print!("[glayout] [info] ");
            println!($($c)*);
        }
    }
}

#[allow(unused_macros)]
macro_rules! warn {
    ($($c:tt)*) => {
        if $crate::utils::log_level::log_level_num() <= 2 {
            print!("[glayout] [warn] ");
            println!($($c)*);
        }
    }
}

#[allow(unused_macros)]
macro_rules! error {
    ($($c:tt)*) => {
        if $crate::utils::log_level::log_level_num() <= 3 {
            print!("[glayout] [error] ");
            println!($($c)*);
        }
    }
}
