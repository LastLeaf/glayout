#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate downcast_rs;

mod utils;
mod lib_interfaces;
pub mod frame;
pub mod tree;
pub mod canvas;

use std::cell::Cell;

lazy_static! {
    static ref SWAP_BUFFER_SIZE: utils::PretendSend<Cell<usize>> = utils::PretendSend::new(Cell::new(4096));
    static ref SWAP_BUFFER: utils::PretendSend<Cell<*mut [u8]>> = utils::PretendSend::new(Cell::new(Box::into_raw(Box::new([0 as u8; 4096]))));
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
pub extern "C" fn callback(callback_ptr: *mut lib_interfaces::Callback, ret_code: i32) {
    let mut callback: Box<lib_interfaces::Callback> = unsafe { Box::from_raw(callback_ptr) };
    callback.callback(ret_code);
}

#[no_mangle]
pub extern "C" fn animation_frame(timestamp: f64) {
    frame::generate(timestamp);
}

#[no_mangle]
pub extern "C" fn set_log_level_num(num: i32) {
    utils::log_level::set_log_level_num(num);
}

pub fn init() {
    lib!(init_lib());
    test();
}

pub fn main_loop() {
    lib!(emscripten_exit_with_live_runtime());
}

// TODO remove test
struct CustomCb (i32);
lib_define_callback!(CustomCb {
    fn callback(&mut self, time: i32) {
        info!("{} Date.now: {}", self.0, time);
    }
});
fn test() {
    lib!(timeout(1000, lib_callback!(CustomCb(666))));
}
