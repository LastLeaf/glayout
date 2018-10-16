#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock, Barrier};
use std::cell::RefCell;
use std::time::{SystemTime, Duration};
use glutin;
use glutin::dpi;
use glutin::GlContext;
use super::Callback;
use super::super::utils::PretendSend;

mod gl;
mod layout_thread;
mod painting_thread;
mod tex_manager;
mod image_manager;
mod font_manager;

use self::gl::Gles2 as Gl;
use self::painting_thread::PaintingCommand;

const DEFAULT_WINDOW_SIZE: (i32, i32) = (1280, 720);
const GL_DRAW_RECT_MAX: i32 = 65536 / 8;
const TEXTURE_MAX: i32 = 16;

lazy_static! {
    static ref MAIN_LOOP: PretendSend<RefCell<MainLoop>> = PretendSend::new(RefCell::new(MainLoop::new()));
    static ref MAIN_LOOP_WINDOWS: Arc<RwLock<HashMap<i32, Mutex<MainLoopWindow>>>> = Arc::new(RwLock::new(HashMap::new()));
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
    redraw_needed: bool,
    keyboard_event_handler: PretendSend<Option<*mut Box<Callback>>>,
    touch_event_handler: PretendSend<Option<*mut Box<Callback>>>,
}

pub fn emscripten_exit_with_live_runtime() {
    main_loop();
}
pub fn init_lib() {
    font_manager::init();
    layout_thread::init();
}
pub fn set_start_fn(f: fn() -> ()) {
    (*MAIN_LOOP).borrow_mut().start_fn = Some(f);
}
pub fn trigger_painting() {
    let windows = MAIN_LOOP_WINDOWS.read().unwrap();
    let barrier_self = Arc::new(Barrier::new(windows.len() + 1));
    for window in windows.iter() {
        let mut window = window.1.lock().unwrap();
        let canvas_index = window.canvas_index;
        let painting_thread = &mut window.painting_thread;
        let barrier = barrier_self.clone();
        painting_thread.append_command(PaintingCommand::CustomCommand(Box::new(move |_ctx, _tex_manager| {
            let w = MAIN_LOOP_WINDOWS.read().unwrap();
            let w = w.get(&canvas_index).unwrap();
            let mut w = w.lock().unwrap();
            if w.redraw_needed {
                w.redraw_needed = false;
                w.gl_window.swap_buffers().unwrap();
            }
            barrier.wait();
        })));
        painting_thread.redraw();
    }
    barrier_self.wait();
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
                    let mut cm = MAIN_LOOP_WINDOWS.read().unwrap();
                    let window_mutex = cm.iter().find(|ref x| x.1.lock().unwrap().window_id == window_id).unwrap().1;
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
        let window = glutin::WindowBuilder::new().with_title("").with_dimensions(dpi::LogicalSize::new(DEFAULT_WINDOW_SIZE.0 as f64, DEFAULT_WINDOW_SIZE.1 as f64));
        let context = glutin::ContextBuilder::new().with_vsync(true);
        let gl_window = glutin::GlWindow::new(window, context, events_loop).unwrap();

        let barrier = Arc::new(Barrier::new(2));
        let barrier_self = barrier.clone();
        let ready_barrier = Arc::new(Barrier::new(2));
        let painting_thread = painting_thread::PaintingThread::new(move || {
            barrier.wait();
            {
                let w = MAIN_LOOP_WINDOWS.read().unwrap();
                let w = w.get(&canvas_index).unwrap();
                let w = w.lock().unwrap();
                unsafe { w.gl_window.make_current().unwrap() };
                let ctx = Box::new(Gl::load_with(|symbol| w.gl_window.get_proc_address(symbol) as *const _));
                ctx
            }
        }, ready_barrier.clone());

        MAIN_LOOP_WINDOWS.write().unwrap().insert(canvas_index, Mutex::new(MainLoopWindow {
            canvas_index,
            window_id: gl_window.window().id(),
            gl_window,
            painting_thread,
            redraw_needed: false,
            keyboard_event_handler: PretendSend::new(None),
            touch_event_handler: PretendSend::new(None),
        }));

        barrier_self.wait();
        ready_barrier.wait();
    }));
}
pub fn unbind_canvas(canvas_index: i32) {
    layout_thread::exec_in_ui_thread(Box::new(move |_events_loop| {
        MAIN_LOOP_WINDOWS.write().unwrap().remove(&canvas_index).unwrap();
    }));
}
pub fn set_title(canvas_index: i32, title: String) {
    let cm = MAIN_LOOP_WINDOWS.read().unwrap();
    cm.get(&canvas_index).unwrap().lock().unwrap().gl_window.set_title(&title);
}

macro_rules! paint {
    ($canvas_index: expr, $f: expr) => {
        let w = MAIN_LOOP_WINDOWS.read().unwrap();
        let w = w.get(&$canvas_index).unwrap();
        let mut w = w.lock().unwrap();
        w.painting_thread.append_command(PaintingCommand::CustomCommand(Box::new($f)));
    }
}
macro_rules! paint_now {
    ($canvas_index: expr, $f: expr) => {
        let w = MAIN_LOOP_WINDOWS.read().unwrap();
        let w = w.get(&$canvas_index).unwrap();
        let mut w = w.lock().unwrap();
        w.painting_thread.exec(Box::new($f));
    }
}

pub fn set_canvas_size(canvas_index: i32, w: i32, h: i32, pixel_ratio: f64) {
    {
        let cm = MAIN_LOOP_WINDOWS.read().unwrap();
        let window = cm.get(&canvas_index).unwrap().lock().unwrap();
        window.gl_window.set_inner_size(dpi::LogicalSize::new(w as f64, h as f64));
        window.gl_window.resize(dpi::PhysicalSize::new(w as f64 * pixel_ratio, h as f64 * pixel_ratio));
    }
    paint_now!(canvas_index, move |ctx, tex_manager| {
        tex_manager.set_tex_draw_size(ctx, w, h, pixel_ratio);
    });
}
pub fn get_device_pixel_ratio(canvas_index: i32) -> f64 {
    let cm = MAIN_LOOP_WINDOWS.read().unwrap();
    let c = cm.get(&canvas_index).unwrap().lock().unwrap();
    c.gl_window.get_hidpi_factor()
}

pub fn set_clear_color(canvas_index: i32, r: f32, g: f32, b: f32, a: f32) {
    paint!(canvas_index, move |ctx, _tex_manager| {
        unsafe { ctx.ClearColor(r, g, b, a) };
    });
}
pub fn clear(canvas_index: i32) {
    paint!(canvas_index, move |ctx, _tex_manager| {
        unsafe { ctx.Clear(gl::COLOR_BUFFER_BIT) };
        let w = MAIN_LOOP_WINDOWS.read().unwrap();
        let w = w.get(&canvas_index).unwrap();
        let mut w = w.lock().unwrap();
        w.redraw_needed = true;
    });
}
pub fn bind_touch_events(canvas_index: i32, cb_ptr: *mut Box<Callback>) {
    let cm = MAIN_LOOP_WINDOWS.read().unwrap();
    let mut c = cm.get(&canvas_index).unwrap().lock().unwrap();
    c.touch_event_handler = PretendSend::new(Some(cb_ptr));
}
pub fn bind_keyboard_events(canvas_index: i32, cb_ptr: *mut Box<Callback>) {
    let cm = MAIN_LOOP_WINDOWS.read().unwrap();
    let mut c = cm.get(&canvas_index).unwrap().lock().unwrap();
    c.keyboard_event_handler = PretendSend::new(Some(cb_ptr));
}

pub fn tex_get_size(canvas_index: i32) -> i32 {
    let w = MAIN_LOOP_WINDOWS.read().unwrap();
    let w = w.get(&canvas_index).unwrap();
    let w = w.lock().unwrap();
    w.painting_thread.get_tex_size()
}
pub fn tex_get_count(canvas_index: i32) -> i32 {
    let w = MAIN_LOOP_WINDOWS.read().unwrap();
    let w = w.get(&canvas_index).unwrap();
    let w = w.lock().unwrap();
    w.painting_thread.get_tex_count()
}
pub fn tex_get_max_draws() -> i32 {
    GL_DRAW_RECT_MAX
}

pub use self::tex_manager::{tex_create_empty, tex_copy, tex_bind_rendering_target, tex_unbind_rendering_target, tex_delete, tex_draw, tex_set_active_texture, tex_draw_end, tex_set_draw_state};
pub use self::image_manager::{image_load_url, image_unload, image_get_natural_width, image_get_natural_height, tex_from_image};
pub use self::font_manager::{text_bind_font_family, text_unbind_font_family, text_set_font, text_get_width, text_to_tex};
