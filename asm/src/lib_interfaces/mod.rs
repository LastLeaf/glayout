#![macro_use]
#![allow(dead_code)]

#[cfg(any(target_arch = "asmjs", target_arch = "wasm32"))]
mod asmjs;
#[cfg(not(any(target_arch = "asmjs", target_arch = "wasm32")))]
mod native;

#[cfg(any(target_arch = "asmjs", target_arch = "wasm32"))]
pub use self::asmjs::*;
#[cfg(not(any(target_arch = "asmjs", target_arch = "wasm32")))]
pub use self::native::*;

pub trait Callback {
    fn callback(&mut self, ret_0: i32, ret_1: i32, ret_2: i32, ret_3: i32) -> bool;
}

pub fn register_callback(callback: Box<Callback>) -> *mut Box<Callback> {
    Box::into_raw(Box::new(callback))
}

#[macro_export]
macro_rules! lib {
    ($x:ident($($y:expr),*)) => {
        unsafe {
            $crate::lib_interfaces::$x($($y),*)
        }
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
