#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate glayout;

use std::ffi::CStr;
use std::os::raw::c_char;

mod utils;
mod test;

pub fn string_from_c_char(char_arr: *const c_char) -> String {
    unsafe {
        CStr::from_ptr(char_arr).to_string_lossy().into_owned()
    }
}

#[no_mangle]
pub extern "C" fn set_test_log_level_num(num: i32) {
    // NOTE static values are not shared between cargo targets, so the log level for glayout_main is separated
    utils::log_level::set_log_level_num(num);
}

#[no_mangle]
pub extern "C" fn load_test_cases() {
    test::init();
}

#[no_mangle]
pub extern "C" fn run_test_case(name_c_char: i32) {
    let name = string_from_c_char(name_c_char as *const c_char);
    log!("Running test case: {}", name);
    run_test_case!(name);
}

fn auto_run() {
    set_test_log_level_num(-1);
    load_test_cases();
    let name = String::from("canvas");
    run_test_case!(name);
}

fn main() {
    glayout::init();
    glayout::main_loop(auto_run);
}
