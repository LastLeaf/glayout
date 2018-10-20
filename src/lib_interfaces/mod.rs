#![macro_use]
#![allow(dead_code)]

use std::time;
use std::cell::Cell;
use super::utils;

#[cfg(any(target_arch = "asmjs", target_arch = "wasm32"))]
mod asmjs;
#[cfg(not(any(target_arch = "asmjs", target_arch = "wasm32")))]
mod native;

#[cfg(any(target_arch = "asmjs", target_arch = "wasm32"))]
pub use self::asmjs::*;
#[cfg(not(any(target_arch = "asmjs", target_arch = "wasm32")))]
pub use self::native::*;

lazy_static! {
    static ref SWAP_BUFFER_SIZE: utils::PretendSend<Cell<usize>> = utils::PretendSend::new(Cell::new(4096));
    static ref SWAP_BUFFER: utils::PretendSend<Cell<*mut [u8]>> = utils::PretendSend::new(Cell::new(Box::into_raw(Box::new([0 as u8; 4096]))));
}

#[cfg(any(target_arch = "asmjs", target_arch = "wasm32"))]
#[macro_export]
macro_rules! lib {
    ($x:ident($($y:expr),*)) => {
        unsafe {
            $crate::lib_interfaces::$x($($y),*)
        }
    }
}
#[cfg(not(any(target_arch = "asmjs", target_arch = "wasm32")))]
#[macro_export]
macro_rules! lib {
    ($x:ident($($y:expr),*)) => {
        $crate::lib_interfaces::$x($($y),*)
    }
}

#[macro_export]
macro_rules! lib_define_callback {
    ($name:ident $fields:tt $block:tt) => {
        struct $name $fields;
        impl $crate::lib_interfaces::Callback for $name $block
    }
}

#[macro_export]
macro_rules! lib_callback {
    ($x:expr) => {
        $crate::lib_interfaces::register_callback(Box::new($x))
    }
}

pub trait Callback {
    fn callback(&mut self, ret_0: i32, ret_1: i32, ret_2: i32, ret_3: i32) -> bool;
}

pub fn register_callback(callback: Box<Callback>) -> *mut Box<Callback> {
    Box::into_raw(Box::new(callback))
}

#[no_mangle]
pub extern "C" fn get_swap_buffer(size: usize) -> i32 {
    if SWAP_BUFFER_SIZE.get() < size {
        SWAP_BUFFER_SIZE.set(size);
        let v = vec![0 as u8; size];
        let boxed = v.into_boxed_slice();
        SWAP_BUFFER.set(Box::into_raw(boxed));
    }
    unsafe {
        let ret = *(**SWAP_BUFFER).as_ptr();
        ret as *const () as i32
    }
}

#[no_mangle]
pub extern "C" fn callback(callback_ptr: *mut Box<Callback>, ret_0: i32, ret_1: i32, ret_2: i32, ret_3: i32) {
    let mut callback: Box<Box<Callback>> = unsafe { Box::from_raw(callback_ptr) };
    if callback.callback(ret_0, ret_1, ret_2, ret_3) {
        Box::into_raw(callback);
    }
}

#[no_mangle]
pub extern "C" fn animation_frame(_timestamp: f64) {
    super::frame::generate(time::Instant::now());
}

#[no_mangle]
pub extern "C" fn set_log_level_num(num: i32) {
    utils::log_level::set_log_level_num(num);
}
