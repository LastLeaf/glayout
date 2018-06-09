#![macro_use]

use std::cell::UnsafeCell;

lazy_static! {
    static ref LOG_LEVEL_NUM: super::PretendSend<UnsafeCell<Box<i32>>> = super::PretendSend::new(UnsafeCell::new(Box::new(0)));
}

#[allow(dead_code)]
pub fn set_log_level_num(num: i32) {
    unsafe {
        **LOG_LEVEL_NUM.get() = num;
    }
}

#[inline]
pub fn get_log_level_num() -> i32 {
    unsafe {
        **LOG_LEVEL_NUM.get()
    }
}

#[allow(unused_macros)]
macro_rules! debug {
    ($($c:tt)*) => {
        if $crate::utils::log_level::get_log_level_num() <= -1 {
            print!("[GLayout] [debug] ");
            println!($($c)*);
        }
    }
}

#[allow(unused_macros)]
macro_rules! log {
    ($($c:tt)*) => {
        if $crate::utils::log_level::get_log_level_num() <= 0 {
            print!("[GLayout] [log] ");
            println!($($c)*);
        }
    }
}

#[allow(unused_macros)]
macro_rules! info {
    ($($c:tt)*) => {
        if $crate::utils::log_level::get_log_level_num() <= 1 {
            print!("[GLayout] [info] ");
            println!($($c)*);
        }
    }
}

#[allow(unused_macros)]
macro_rules! warn {
    ($($c:tt)*) => {
        if $crate::utils::log_level::get_log_level_num() <= 2 {
            print!("[GLayout] [warn] ");
            println!($($c)*);
        }
    }
}

#[allow(unused_macros)]
macro_rules! error {
    ($($c:tt)*) => {
        if $crate::utils::log_level::get_log_level_num() <= 3 {
            print!("[GLayout] [error] ");
            println!($($c)*);
        }
    }
}
