#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::cell::RefCell;
use std::os::raw::c_char;
use std::time::{SystemTime, Duration};
use glutin;
use glutin::dpi;
use glutin::GlContext;
use super::Callback;
use super::super::utils::PretendSend;

mod gl;
mod layout_thread;

lazy_static! {
    static ref MAIN_LOOP: PretendSend<RefCell<MainLoop>> = PretendSend::new(RefCell::new(MainLoop::new()));
    static ref MAIN_LOOP_WINDOWS: Arc<RwLock<HashMap<i32, Mutex<MainLoopWindow>>>> = Arc::new(RwLock::new(HashMap::new()));
}

struct MainLoop {
    events_loop: glutin::EventsLoop,
    window_size_listener: Option<*mut Box<Callback>>,
}

impl MainLoop {
    fn new() -> Self {
        MainLoop {
            events_loop: glutin::EventsLoop::new(),
            window_size_listener: None,
        }
    }
}

struct MainLoopWindow {
    canvas_index: i32,
    window_id: glutin::WindowId,
    gl_window: glutin::GlWindow,
    ctx: gl::Gles2,
}

pub fn emscripten_exit_with_live_runtime() {
    main_loop();
}

pub fn init_lib() {
    layout_thread::init();
}
fn main_loop() {
    // listening to user events
    let events_loop = &mut (*MAIN_LOOP).borrow_mut().events_loop;
    let mut running = true;
    while running {
        layout_thread::wakeup(); // TODO
        events_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent { event, window_id } => {
                    let mut cm = MAIN_LOOP_WINDOWS.write().unwrap();
                    let window_mutex = cm.iter_mut().find(|ref x| x.1.lock().unwrap().window_id == window_id).unwrap().1;
                    let window = window_mutex.lock().unwrap();
                    match event {
                        glutin::WindowEvent::CloseRequested => running = false,
                        glutin::WindowEvent::Resized(logical_size) => {
                            let dpi_factor = window.gl_window.get_hidpi_factor();
                            window.gl_window.resize(logical_size.to_physical(dpi_factor));
                            // TODO push to event queue
                        },
                        _ => ()
                    }
                },
                _ => ()
            }
        });

        // gl_window.swap_buffers().unwrap();
    }
}
pub fn set_window_size_listener(cb_ptr: *mut Box<Callback>) {
    (*MAIN_LOOP).borrow_mut().window_size_listener = Some(cb_ptr);
}
pub fn get_window_width() -> i32 {
    1
}
pub fn get_window_height() -> i32 {
    1
}
pub fn timeout(ms: i32, cb_ptr: *mut Box<Callback>) {
    layout_thread::push_event(
        SystemTime::now() + Duration::new((ms / 1000) as u64, (ms % 1000 * 1000000) as u32),
        layout_thread::EventDetail::TimeoutEvent,
        move |_detail| {
            super::callback(cb_ptr, 0, 0, 0, 0);
        }
    );
}
pub fn enable_animation_frame() {
    unimplemented!();
}
pub fn disable_animation_frame() {
    unimplemented!();
}

pub fn bind_canvas(canvas_index: i32) {
    let window = glutin::WindowBuilder::new().with_title("");
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let gl_window = glutin::GlWindow::new(window, context, &(*MAIN_LOOP).borrow_mut().events_loop).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
    }

    let ctx = unsafe {
        let ctx = gl::Gles2::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        ctx.ClearColor(0.0, 0.0, 0.0, 0.0);
        ctx
    };

    MAIN_LOOP_WINDOWS.write().unwrap().insert(canvas_index, Mutex::new(MainLoopWindow {
        canvas_index,
        window_id: gl_window.window().id(),
        gl_window,
        ctx,
    }));
}
pub fn unbind_canvas(canvas_index: i32) {
    MAIN_LOOP_WINDOWS.write().unwrap().remove(&canvas_index).unwrap();
}
pub fn set_title(canvas_index: i32, title: String) {
    let cm = MAIN_LOOP_WINDOWS.write().unwrap();
    cm.get(&canvas_index).unwrap().lock().unwrap().gl_window.set_title(&title);
}
pub fn set_canvas_size(canvas_index: i32, w: i32, h: i32, pixel_ratio: f64) {
    let cm = MAIN_LOOP_WINDOWS.write().unwrap();
    cm.get(&canvas_index).unwrap().lock().unwrap().gl_window.set_inner_size(dpi::LogicalSize::new(w as f64, h as f64));
}
pub fn get_device_pixel_ratio(canvas_index: i32) -> f64 {
    let cm = MAIN_LOOP_WINDOWS.write().unwrap();
    let c = cm.get(&canvas_index).unwrap().lock().unwrap();
    c.gl_window.get_hidpi_factor()
}

macro_rules! ctx_from_canvas_index {
    ($x:ident, $y:ident) => {
        let cm = MAIN_LOOP_WINDOWS.write().unwrap();
        let mut c = cm.get(&$y).unwrap().lock().unwrap();
        let $x = &mut c.ctx;
    }
}
pub fn set_clear_color(canvas_index: i32, r: f32, g: f32, b: f32, a: f32) {
    ctx_from_canvas_index!(ctx, canvas_index);
    unsafe { ctx.ClearColor(r, g, b, a) };
}
pub fn clear(canvas_index: i32) {
    ctx_from_canvas_index!(ctx, canvas_index);
    unsafe { ctx.Clear(gl::COLOR_BUFFER_BIT) };
}
pub fn bind_touch_events(canvas_index: i32, cbPtr: *mut Box<Callback>) {
    unimplemented!();
}
pub fn bind_keyboard_events(canvas_index: i32, cbPtr: *mut Box<Callback>) {
    unimplemented!();
}

pub fn tex_get_size(canvas_index: i32) -> i32 {
    unimplemented!();
}
pub fn tex_get_count(canvas_index: i32) -> i32 {
    unimplemented!();
}
pub fn tex_get_max_draws() -> i32 {
    unimplemented!();
}
pub fn tex_create_empty(canvas_index: i32, texId: i32, width: i32, height: i32) {
    unimplemented!();
}
pub fn tex_copy(canvas_index: i32, destTexId: i32, destLeft: i32, destTop: i32, srcLeft: i32, srcTop: i32, width: i32, height: i32) {
    unimplemented!();
}
pub fn tex_bind_rendering_target(canvas_index: i32, texId: i32, width: i32, height: i32) {
    unimplemented!();
}
pub fn tex_unbind_rendering_target(canvas_index: i32) {
    unimplemented!();
}
pub fn tex_delete(canvas_index: i32, texId: i32) {
    unimplemented!();
}
pub fn tex_draw(canvas_index: i32, drawIndex: i32, texShaderIndex: i32, normalizedTexX: f64, normalizedTexY: f64, normalizedTexW: f64, normalizedTexH: f64, x: f64, y: f64, w: f64, h: f64) {
    unimplemented!();
}
pub fn tex_set_active_texture(canvas_index: i32, texShaderIndex: i32, texId: i32) {
    unimplemented!();
}
pub fn tex_draw_end(canvas_index: i32, drawCount: i32) {
    unimplemented!();
}
pub fn tex_set_draw_state(canvas_index: i32, colorR: f32, colorG: f32, colorB: f32, colorA: f32, alpha: f32) {
    unimplemented!();
}

pub fn image_load_url(id: i32, url: *mut c_char, cbPtr: *mut Box<Callback>) {
    unimplemented!();
}
pub fn image_unload(id: i32) {
    unimplemented!();
}
pub fn image_get_natural_width(id: i32) -> i32 {
    unimplemented!();
}
pub fn image_get_natural_height(id: i32) -> i32 {
    unimplemented!();
}
pub fn tex_from_image(canvas_index: i32, texId: i32, imgId: i32) {
    unimplemented!();
}

pub fn text_bind_font_family(id: i32, fontFamily: *mut c_char) {
    unimplemented!();
}
pub fn text_unbind_font_family(id: i32) {
    unimplemented!();
}
pub fn text_set_font(fontSize: i32, lineHeight: i32, fontFamilyId: i32, italic: i32, bold: i32) {
    unimplemented!();
}
pub fn text_get_width(text: *mut c_char) -> f64 {
    unimplemented!();
}
pub fn text_to_tex(canvas_index: i32, texId: i32, texLeft: i32, texTop: i32, text: *mut c_char, width: i32, height: i32, lineHeight: i32) {
    unimplemented!();
}