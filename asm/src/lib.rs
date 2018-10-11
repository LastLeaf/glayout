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

use std::sync::{Arc, Mutex};

lazy_static! {
    static ref WINDOW_SIZE: Arc<Mutex<(f64, f64)>> = Arc::new(Mutex::new((0., 0.)));
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
