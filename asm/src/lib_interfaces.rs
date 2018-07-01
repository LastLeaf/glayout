#![macro_use]
#![allow(improper_ctypes, dead_code)]

use std::os::raw::c_char;

extern {
    pub fn emscripten_exit_with_live_runtime();

    pub fn init_lib();
    pub fn timeout(ms: i32, cbPtr: *mut Box<Callback>);
    pub fn enable_animation_frame();
    pub fn disable_animation_frame();

    pub fn bind_canvas(canvasIndex: i32);
    pub fn unbind_canvas(canvasIndex: i32);
    pub fn set_canvas_size(canvasIndex: i32, w: i32, h: i32, pixel_ratio: f64);
    pub fn get_device_pixel_ratio() -> f64;
    pub fn set_clear_color(canvasIndex: i32, r: f32, g: f32, b: f32, a: f32);
    pub fn clear(canvasIndex: i32);

    pub fn tex_get_size(canvasIndex: i32) -> i32;
    pub fn tex_get_count(canvasIndex: i32) -> i32;
    pub fn tex_get_max_draws() -> i32;
    pub fn tex_create_empty(canvasIndex: i32, texId: i32, width: i32, height: i32);
    pub fn tex_copy(canvasIndex: i32, destTexId: i32, destLeft: i32, destTop: i32, srcLeft: i32, srcTop: i32, width: i32, height: i32);
    pub fn tex_bind_rendering_target(canvasIndex: i32, texId: i32, width: i32, height: i32);
    pub fn tex_unbind_rendering_target(canvasIndex: i32);
    pub fn tex_delete(canvasIndex: i32, texId: i32);
    pub fn tex_draw(canvasIndex: i32, drawIndex: i32, texShaderIndex: i32, normalizedTexX: f64, normalizedTexY: f64, normalizedTexW: f64, normalizedTexH: f64, x: f64, y: f64, w: f64, h: f64);
    pub fn tex_set_active_texture(canvasIndex: i32, texShaderIndex: i32, texId: i32);
    pub fn tex_draw_end(canvasIndex: i32, drawCount: i32);

    pub fn image_load_url(id: i32, url: *mut c_char, cbPtr: *mut Box<Callback>);
    pub fn image_unload(id: i32);
    pub fn image_get_natural_width(id: i32) -> i32;
    pub fn image_get_natural_height(id: i32) -> i32;
    pub fn tex_from_image(canvasIndex: i32, texId: i32, imgId: i32);

    pub fn text_bind_font_family(id: i32, fontFamily: *mut c_char);
    pub fn text_unbind_font_family(id: i32);
    pub fn text_set_font(fontSize: i32, lineHeight: i32, fontFamilyId: i32, italic: i32, bold: i32);
    pub fn text_get_width(text: *mut c_char) -> f64;
    pub fn text_to_tex(canvasIndex: i32, texId: i32, texLeft: i32, texTop: i32, text: *mut c_char, width: i32, height: i32, lineHeight: i32);
}

pub trait Callback {
    fn callback(&mut self, ret_code: i32);
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
    ($x:ident $y:tt) => {
        impl $crate::lib_interfaces::Callback for $x $y
    }
}

#[macro_export]
macro_rules! lib_callback {
    ($x:expr) => {
        $crate::lib_interfaces::register_callback(Box::new($x))
    }
}
