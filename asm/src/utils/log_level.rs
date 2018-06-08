#![macro_use]

use std::cell::Cell;

lazy_static! {
    static ref LOG_LEVEL_NUM: super::PretendSend<Cell<i32>> = super::PretendSend::new(Cell::new(0));
}

pub fn set_log_level_num(num: i32) {
    (*LOG_LEVEL_NUM).set(num);
}

#[inline]
pub fn get_log_level_num() -> i32 {
    (*LOG_LEVEL_NUM).get()
}

macro_rules! debug {
    ($($c:tt)*) => {
        if $crate::utils::log_level::get_log_level_num() <= -1 {
            print!("[GLayout] [debug] ");
            println!($($c)*);
        }
    }
}

macro_rules! log {
    ($($c:tt)*) => {
        if $crate::utils::log_level::get_log_level_num() <= 0 {
            print!("[GLayout] [log] ");
            println!($($c)*);
        }
    }
}

macro_rules! info {
    ($($c:tt)*) => {
        if $crate::utils::log_level::get_log_level_num() <= 1 {
            print!("[GLayout] [info] ");
            println!($($c)*);
        }
    }
}

macro_rules! warn {
    ($($c:tt)*) => {
        if $crate::utils::log_level::get_log_level_num() <= 2 {
            print!("[GLayout] [warn] ");
            println!($($c)*);
        }
    }
}

macro_rules! error {
    ($($c:tt)*) => {
        if $crate::utils::log_level::get_log_level_num() <= 3 {
            print!("[GLayout] [error] ");
            println!($($c)*);
        }
    }
}
