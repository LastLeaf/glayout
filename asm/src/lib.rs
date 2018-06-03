#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate downcast_rs;

mod utils;
mod lib_interfaces;
pub mod frame;
pub mod tree;
pub mod canvas;

#[no_mangle]
pub extern "C" fn callback(callback_ptr: *mut lib_interfaces::Callback, ret_code: i32) {
    let mut callback: Box<lib_interfaces::Callback> = unsafe { Box::from_raw(callback_ptr) };
    callback.callback(ret_code);
}

#[no_mangle]
pub extern "C" fn animation_frame(timestamp: f64) {
    frame::generate(timestamp);
}

pub fn init() {
    lib!(init_lib());
    test();
}

pub fn main_loop() {
    lib!(emscripten_exit_with_live_runtime());
}

// test
struct CustomCb (i32);
lib_define_callback!(CustomCb {
    fn callback(&mut self, time: i32) {
        println!("{} Date.now: {}", self.0, time);
    }
});
fn test() {
    lib!(timeout(1000, lib_callback!(CustomCb(666))));
}
