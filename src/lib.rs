#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate downcast_rs;
extern crate cssparser;
#[cfg(not(any(target_arch = "asmjs", target_arch = "wasm32")))]
extern crate glutin;
#[cfg(not(any(target_arch = "asmjs", target_arch = "wasm32")))]
extern crate image;
#[cfg(not(any(target_arch = "asmjs", target_arch = "wasm32")))]
extern crate euclid;
#[cfg(not(any(target_arch = "asmjs", target_arch = "wasm32")))]
extern crate font_kit;

mod utils;
pub mod lib_interfaces;
pub mod frame;
pub mod tree;
pub mod canvas;

use std::time;

/// Set the log level number
/// * `-1` debug
/// * `0` log
/// * `1` info
/// * `2` warn
/// * `3` error
pub fn set_log_level_num(level: i32) {
    utils::log_level::set_log_level_num(level);
}

pub fn log_with_level(str: String, level: i32) {
    lib!(log_with_level(str, level));
}

lib_define_callback!(TimeoutCallback (Box<Fn() + 'static>) {
    fn callback(&mut self, _: i32, _: i32, _: i32, _: i32) -> bool {
        self.0();
        false
    }
});

pub fn set_timeout<F>(f: F, dur: time::Duration) where F: Fn() + 'static {
    let ms = dur.as_secs() as i32 * 1000 + (dur.subsec_nanos() as f64 / 1_000_000.).ceil() as i32;
    lib!(timeout(ms, lib_callback!(TimeoutCallback(Box::new(f)))));
}

pub fn init() {
    lib!(init_lib());
}

pub fn main_loop(f: fn() -> ()) {
    lib!(main_loop(f));
}
