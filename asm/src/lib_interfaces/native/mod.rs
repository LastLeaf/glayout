#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::cell::RefCell;
use std::os::raw::c_char;
use std::ffi::CStr;
use std::path::Path;
use std::time::{SystemTime, Duration};
use std::thread;
use image;
use glutin;
use glutin::dpi;
use glutin::GlContext;
use super::Callback;
use super::super::utils::PretendSend;

mod gl;
mod layout_thread;
mod painting_thread;

use self::painting_thread::PaintingCommand;

const GL_DRAW_RECT_MAX: i32 = 65536 / 8;
const TEXTURE_MAX: i32 = 16;

lazy_static! {
    static ref MAIN_LOOP: PretendSend<RefCell<MainLoop>> = PretendSend::new(RefCell::new(MainLoop::new()));
    static ref MAIN_LOOP_WINDOWS: Arc<RwLock<HashMap<i32, Mutex<MainLoopWindow>>>> = Arc::new(RwLock::new(HashMap::new()));
    static ref IMAGES: Arc<Mutex<HashMap<i32, (u32, u32, Vec<u8>)>>> = Arc::new(Mutex::new(HashMap::new()));
}

struct MainLoop {
    events_loop: glutin::EventsLoop,
    start_fn: Option<fn() -> ()>,
}

impl MainLoop {
    fn new() -> Self {
        MainLoop {
            events_loop: glutin::EventsLoop::new(),
            start_fn: None,
        }
    }
}

struct MainLoopWindow {
    canvas_index: i32,
    window_id: glutin::WindowId,
    gl_window: glutin::GlWindow,
    painting_thread: painting_thread::PaintingThread,
    max_tex_size: i32,
    max_tex_count: i32,
    keyboard_event_handler: PretendSend<Option<*mut Box<Callback>>>,
    touch_event_handler: PretendSend<Option<*mut Box<Callback>>>,
}

pub fn emscripten_exit_with_live_runtime() {
    main_loop();
}
pub fn init_lib() {
    layout_thread::init();
}
pub fn set_start_fn(f: fn() -> ()) {
    (*MAIN_LOOP).borrow_mut().start_fn = Some(f);
}
pub fn trigger_painting() {
    for window in MAIN_LOOP_WINDOWS.write().unwrap().iter_mut() {
        let mut window = window.1.lock().unwrap();
        let canvas_index = window.canvas_index;
        let painting_thread = &mut window.painting_thread;
        painting_thread.append_command(PaintingCommand::CustomCommand(Box::new(move |_ctx| {
            let w = MAIN_LOOP_WINDOWS.write().unwrap();
            let w = w.get(&canvas_index).unwrap();
            let w = w.lock().unwrap();
            w.gl_window.swap_buffers().unwrap();
        })));
        painting_thread.redraw();
    }
}
fn main_loop() {
    // listening to user events
    let mut main_loop = (*MAIN_LOOP).borrow_mut();
    match main_loop.start_fn.take() {
        None => { panic!() },
        Some(f) => {
            layout_thread::push_event(SystemTime::now(), layout_thread::EventDetail::TimeoutEvent,
            move |_time, _detail| {
                f()
            })
        }
    }
    let events_loop = &mut main_loop.events_loop;
    layout_thread::set_ui_thread_handle(events_loop.create_proxy());
    loop {
        layout_thread::wakeup();
        let mut running = true;
        events_loop.run_forever(|event| {
            match event {
                glutin::Event::Awakened => {
                    return glutin::ControlFlow::Break;
                },
                glutin::Event::WindowEvent { event, window_id } => {
                    let mut cm = MAIN_LOOP_WINDOWS.write().unwrap();
                    let window_mutex = cm.iter_mut().find(|ref x| x.1.lock().unwrap().window_id == window_id).unwrap().1;
                    let window = window_mutex.lock().unwrap();
                    match event {
                        glutin::WindowEvent::CloseRequested => {
                            // TODO
                            running = false;
                            return glutin::ControlFlow::Break;
                        },
                        glutin::WindowEvent::Resized(logical_size) => {
                            // TODO should do nothing
                            let dpi_factor = window.gl_window.get_hidpi_factor();
                            window.gl_window.resize(logical_size.to_physical(dpi_factor));
                        },
                        _ => {
                            layout_thread::push_event(
                                SystemTime::now(),
                                layout_thread::EventDetail::WindowEvent(event, window.canvas_index),
                                move |_time, detail| {
                                    match detail {
                                        layout_thread::EventDetail::WindowEvent(event, canvas_index) => {
                                            // TODO
                                        },
                                        _ => {
                                            panic!()
                                        }
                                    }
                                }
                            );
                        }
                    }

                },
                _ => ()
            }
            layout_thread::wakeup();
            glutin::ControlFlow::Continue
        });
        if running {
            layout_thread::exec_ui_thread_task(events_loop);
        } else {
            break;
        }
    }
}
pub fn set_window_size_listener(cb_ptr: *mut Box<Callback>) {
    // TODO redesign window size change fn
    // (*MAIN_LOOP).borrow_mut().window_size_listener = Some(cb_ptr);
}
pub fn get_window_width() -> i32 {
    1
}
pub fn get_window_height() -> i32 {
    1
}
pub fn timeout(ms: i32, cb_ptr: *mut Box<Callback>) {
    layout_thread::push_event_from_layout_thread(
        SystemTime::now() + Duration::new((ms / 1000) as u64, (ms % 1000 * 1000000) as u32),
        layout_thread::EventDetail::TimeoutEvent,
        move |_time, _detail| {
            super::callback(cb_ptr, 0, 0, 0, 0);
        }
    );
    layout_thread::wakeup();
}
pub fn enable_animation_frame() {
    layout_thread::set_animation_frame_enabled(true);
}
pub fn disable_animation_frame() {
    layout_thread::set_animation_frame_enabled(false);
}

pub fn bind_canvas(canvas_index: i32) {
    layout_thread::exec_in_ui_thread(Box::new(move |events_loop| {
        let window = glutin::WindowBuilder::new().with_title("").with_dimensions(dpi::LogicalSize::new(1280., 720.));
        let context = glutin::ContextBuilder::new().with_vsync(true);
        let gl_window = glutin::GlWindow::new(window, context, events_loop).unwrap();
        let ctx = gl::Gles2::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

        let max_tex_size = unsafe {
            let mut ret = 4096;
            ctx.GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut ret as *mut i32);
            ret
        };
        let max_tex_count = unsafe {
            let mut ret = 16;
            ctx.GetIntegerv(gl::MAX_TEXTURE_IMAGE_UNITS, &mut ret as *mut i32);
            ret
        };

        let mut painting_thread = painting_thread::PaintingThread::new(ctx);
        painting_thread.append_command(PaintingCommand::CustomCommand(Box::new(move |_ctx| {
            let w = MAIN_LOOP_WINDOWS.write().unwrap();
            let w = w.get(&canvas_index).unwrap();
            let w = w.lock().unwrap();
            unsafe { w.gl_window.make_current().unwrap() };
        })));

        MAIN_LOOP_WINDOWS.write().unwrap().insert(canvas_index, Mutex::new(MainLoopWindow {
            canvas_index,
            window_id: gl_window.window().id(),
            gl_window,
            painting_thread,
            max_tex_size,
            max_tex_count,
            keyboard_event_handler: PretendSend::new(None),
            touch_event_handler: PretendSend::new(None),
        }));
    }));
}
pub fn unbind_canvas(canvas_index: i32) {
    layout_thread::exec_in_ui_thread(Box::new(move |_events_loop| {
        MAIN_LOOP_WINDOWS.write().unwrap().remove(&canvas_index).unwrap();
    }));
}
pub fn set_title(canvas_index: i32, title: String) {
    let cm = MAIN_LOOP_WINDOWS.write().unwrap();
    cm.get(&canvas_index).unwrap().lock().unwrap().gl_window.set_title(&title);
}
pub fn set_canvas_size(canvas_index: i32, w: i32, h: i32, pixel_ratio: f64) {
    let cm = MAIN_LOOP_WINDOWS.write().unwrap();
    let window = cm.get(&canvas_index).unwrap().lock().unwrap();
    window.gl_window.set_inner_size(dpi::LogicalSize::new(w as f64, h as f64));
    window.gl_window.resize(dpi::PhysicalSize::new(w as f64 * pixel_ratio, h as f64 * pixel_ratio));
}
pub fn get_device_pixel_ratio(canvas_index: i32) -> f64 {
    let cm = MAIN_LOOP_WINDOWS.write().unwrap();
    let c = cm.get(&canvas_index).unwrap().lock().unwrap();
    c.gl_window.get_hidpi_factor()
}

macro_rules! paint {
    ($canvas_index: expr, $f: expr) => {
        let w = MAIN_LOOP_WINDOWS.read().unwrap();
        let w = w.get(&$canvas_index).unwrap();
        let mut w = w.lock().unwrap();
        w.painting_thread.append_command(PaintingCommand::CustomCommand(Box::new($f)));
    }
}

pub fn set_clear_color(canvas_index: i32, r: f32, g: f32, b: f32, a: f32) {
    paint!(canvas_index, move |ctx| {
        unsafe { ctx.ClearColor(r, g, b, a) };
    });
}
pub fn clear(canvas_index: i32) {
    paint!(canvas_index, move |ctx| {
        unsafe { ctx.Clear(gl::COLOR_BUFFER_BIT) };
    });
}
pub fn bind_touch_events(canvas_index: i32, cb_ptr: *mut Box<Callback>) {
    let cm = MAIN_LOOP_WINDOWS.write().unwrap();
    let mut c = cm.get(&canvas_index).unwrap().lock().unwrap();
    c.touch_event_handler = PretendSend::new(Some(cb_ptr));
}
pub fn bind_keyboard_events(canvas_index: i32, cb_ptr: *mut Box<Callback>) {
    let cm = MAIN_LOOP_WINDOWS.write().unwrap();
    let mut c = cm.get(&canvas_index).unwrap().lock().unwrap();
    c.keyboard_event_handler = PretendSend::new(Some(cb_ptr));
}

pub fn tex_get_size(canvas_index: i32) -> i32 {
    let w = MAIN_LOOP_WINDOWS.read().unwrap();
    let w = w.get(&canvas_index).unwrap();
    let w = w.lock().unwrap();
    w.max_tex_size
}
pub fn tex_get_count(canvas_index: i32) -> i32 {
    let w = MAIN_LOOP_WINDOWS.read().unwrap();
    let w = w.get(&canvas_index).unwrap();
    let w = w.lock().unwrap();
    w.max_tex_count
}
pub fn tex_get_max_draws() -> i32 {
    GL_DRAW_RECT_MAX
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

pub fn image_load_url(id: i32, url: *mut c_char, cb_ptr: *mut Box<Callback>) {
    let url = unsafe { CStr::from_ptr(url) };
    let url = Path::new(url.to_str().unwrap());
    let cb_ptr = PretendSend::new(cb_ptr);
    thread::spawn(move || {
        let rgba_image = image::open(url).unwrap().to_rgba();
        let image_info = (rgba_image.width(), rgba_image.height(), rgba_image.into_raw());
        IMAGES.lock().unwrap().insert(id, image_info);
        layout_thread::push_event(SystemTime::now(), layout_thread::EventDetail::ImageLoadEvent, move |_time, _detail| {
            super::callback(*cb_ptr, 0, 0, 0, 0);
        })
    });
}
pub fn image_unload(id: i32) {
    IMAGES.lock().unwrap().remove(&id);
}
pub fn image_get_natural_width(id: i32) -> i32 {
    IMAGES.lock().unwrap().get(&id).unwrap().0 as i32
}
pub fn image_get_natural_height(id: i32) -> i32 {
    IMAGES.lock().unwrap().get(&id).unwrap().1 as i32
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
