#![macro_use]

mod pretend_send;
pub type PretendSend<T> = pretend_send::PretendSend<T>;

pub mod log_level;
