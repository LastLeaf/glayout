#[macro_use]
extern crate lazy_static;

extern crate glayout;

use std::ffi::CStr;
use std::os::raw::c_char;

mod utils;
mod test;

pub fn get_string_from_c_char(char_arr: *const c_char) -> String {
    unsafe {
        CStr::from_ptr(char_arr).to_string_lossy().into_owned()
    }
}

#[no_mangle]
pub extern "C" fn run_test_case(name_c_char: *const c_char) {
    let name = get_string_from_c_char(name_c_char);
    run_test_case!(name);
}

fn main() {
    glayout::init();
    test::init();
    glayout::main_loop();
}
