extern crate glayout;

#[no_mangle]
pub extern "C" fn test() -> i32 {
    let mut err = 0;
    err += glayout::frame::test::test();
    err += glayout::canvas::test::test();
    return err;
}

fn main() {
    glayout::init();
    test();
    glayout::main_loop();
}
