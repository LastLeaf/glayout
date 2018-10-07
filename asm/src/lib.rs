#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate downcast_rs;
extern crate glutin;

mod utils;
pub mod lib_interfaces;
pub mod frame;
pub mod tree;
pub mod canvas;

use std::cell::Cell;

lazy_static! {
    static ref SWAP_BUFFER_SIZE: utils::PretendSend<Cell<usize>> = utils::PretendSend::new(Cell::new(4096));
    static ref SWAP_BUFFER: utils::PretendSend<Cell<*mut [u8]>> = utils::PretendSend::new(Cell::new(Box::into_raw(Box::new([0 as u8; 4096]))));
    static ref WINDOW_SIZE: utils::PretendSend<Cell<(f64, f64)>> = utils::PretendSend::new(Cell::new((0., 0.)));
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
pub extern "C" fn callback(callback_ptr: *mut lib_interfaces::Callback, ret_0: i32, ret_1: i32, ret_2: i32, ret_3: i32) {
    let mut callback: Box<lib_interfaces::Callback> = unsafe { Box::from_raw(callback_ptr) };
    if callback.callback(ret_0, ret_1, ret_2, ret_3) {
        Box::into_raw(callback);
    }
}

#[no_mangle]
pub extern "C" fn animation_frame(timestamp: f64) {
    frame::generate(timestamp);
}

#[no_mangle]
pub extern "C" fn set_log_level_num(num: i32) {
    utils::log_level::set_log_level_num(num);
}

pub fn window_size() -> (f64, f64) {
    WINDOW_SIZE.get()
}

lib_define_callback!(WindowSizeCallback () {
    fn callback(&mut self, _combined_size: i32, _: i32, _: i32, _: i32) -> bool {
        WINDOW_SIZE.set((lib!(get_window_width()) as f64, lib!(get_window_height()) as f64));
        true
    }
});

pub fn init() {
    lib!(init_lib());
    WINDOW_SIZE.set((lib!(get_window_width()) as f64, lib!(get_window_width()) as f64));
    lib!(set_window_size_listener(lib_callback!(WindowSizeCallback())));
}

pub fn main_loop() {
    lib!(emscripten_exit_with_live_runtime());
}
