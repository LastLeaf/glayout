#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate downcast_rs;
extern crate glutin;
extern crate image;
extern crate euclid;
extern crate font_kit;

mod utils;
pub mod lib_interfaces;
pub mod frame;
pub mod tree;
pub mod canvas;

use std::sync::{Arc, Mutex};
use std::time;

lazy_static! {
    static ref WINDOW_SIZE: Arc<Mutex<(f64, f64)>> = Arc::new(Mutex::new((0., 0.)));
}

/// Set the log level number
/// * `-1` debug
/// * `0` log
/// * `1` info
/// * `2` warn
/// * `3` error
pub fn set_log_level_num(level: i32) {
    utils::log_level::set_log_level_num(level);
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

pub fn window_size() -> (f64, f64) {
    *WINDOW_SIZE.lock().unwrap()
}

lib_define_callback!(WindowSizeCallback () {
    fn callback(&mut self, _combined_size: i32, _: i32, _: i32, _: i32) -> bool {
        *WINDOW_SIZE.lock().unwrap() = (lib!(get_window_width()) as f64, lib!(get_window_height()) as f64);
        true
    }
});

pub fn init() {
    lib!(init_lib());
    *WINDOW_SIZE.lock().unwrap() = (lib!(get_window_width()) as f64, lib!(get_window_height()) as f64);
    lib!(set_window_size_listener(lib_callback!(WindowSizeCallback())));
}

pub fn main_loop(f: fn() -> ()) {
    lib!(set_start_fn(f));
    lib!(emscripten_exit_with_live_runtime());
}
