use std::rc::Rc;
use std::cell::RefCell;
use super::frame;
use rc_forest::{Forest, ForestNodeRc};

pub mod element;
mod config;
mod character;
mod resource;

pub type CanvasConfig = config::CanvasConfig;
pub type Element = element::Element;
pub type Empty = element::Empty;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct KeyDescriptor {
    pub key_code: i32,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
    pub is_down: bool,
}

pub struct CanvasContext {
    canvas_config: Rc<CanvasConfig>,
    root_node: ForestNodeRc<Element>,
    need_redraw: bool,
    touching: bool,
    touch_point: element::Point,
    last_key: KeyDescriptor,
}

pub struct Canvas {
    context: Rc<RefCell<CanvasContext>>,
    frame_fn: frame::FrameCallback,
}

impl Canvas {
    pub fn new(index: i32) -> Self {
        lib!(bind_canvas(index));
        let canvas_config = Rc::new(CanvasConfig::new(
            index,
            lib!(tex_get_size(index)) as i32,
            lib!(tex_get_count(index)) as i32,
            lib!(tex_get_max_draws()) as i32,
            lib!(get_device_pixel_ratio(index)) as f64
        ));
        log!("Canvas binded: {}", index);
        let root_node = ForestNodeRc::new(&mut Forest::new(), Element::new(&canvas_config, Box::new(Empty::new(&canvas_config))));
        let ctx = Rc::new(RefCell::new(CanvasContext {
            canvas_config,
            root_node,
            need_redraw: false,
            touching: false,
            touch_point: element::Point::new(0., 0.),
            last_key: Default::default(),
        }));
        let frame_ctx = ctx.clone();
        let frame_fn = frame::FrameCallback::new(Box::new(move |_time| {
            frame_ctx.borrow_mut().generate_frame();
            true
        }));
        frame::bind(frame_fn.clone(), frame::FramePriority::Low); // FIXME only bind when neccessary
        lib!(bind_touch_events(index, lib_callback!(TouchEventCallback(ctx.clone()))));
        lib!(bind_keyboard_events(index, lib_callback!(KeyboardEventCallback(ctx.clone()))));
        lib!(bind_canvas_size_change(index, lib_callback!(CanvasSizeChangeCallback(ctx.clone()))));
        return Canvas {
            context: ctx,
            frame_fn,
        };
    }
    #[inline]
    pub fn destroy(&mut self) {
        frame::unbind(self.frame_fn.clone(), frame::FramePriority::Low);
    }
    #[inline]
    pub fn context(&self) -> Rc<RefCell<CanvasContext>> {
        self.context.clone()
    }
    #[inline]
    pub fn ctx<F>(&mut self, f: F) where F: Fn(&mut CanvasContext) {
        f(&mut *self.context.borrow_mut());
    }
}

impl Drop for CanvasContext {
    fn drop(&mut self) {
        log!("Canvas unbinded: {}", self.canvas_config.index);
        lib!(unbind_canvas(self.canvas_config.index));
    }
}

impl CanvasContext {
    #[inline]
    pub fn canvas_config(&mut self) -> Rc<CanvasConfig> {
        self.canvas_config.clone()
    }
    fn set_canvas_size_inner(&mut self, w: i32, h: i32, pixel_ratio: f64, update_logical_size: bool) {
        log!("Canvas size changed: {}", self.canvas_config.index);
        self.canvas_config.canvas_size.set(element::Size::new(w as f64, h as f64));
        lib!(set_canvas_size(self.canvas_config.index, w, h, pixel_ratio, update_logical_size as i32));
        self.root_node.borrow_mut().mark_layout_dirty();
    }
    pub fn set_canvas_size(&mut self, w: i32, h: i32, pixel_ratio: f64) {
        self.set_canvas_size_inner(w, h, pixel_ratio, true)
    }
    #[inline]
    pub fn device_pixel_ratio(&self) -> f64 {
        lib!(get_device_pixel_ratio(self.canvas_config.index))
    }
    #[inline]
    pub fn canvas_size(&self) -> (i32, i32) {
        (lib!(get_canvas_width(self.canvas_config.index)), lib!(get_canvas_height(self.canvas_config.index)))
    }
    pub fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.canvas_config.set_clear_color((r, g, b, a));
        lib!(set_clear_color(self.canvas_config.index, r, g, b, a));
    }
    pub fn clear(&mut self) {
        let (r, g, b, a) = self.canvas_config.clear_color();
        lib!(set_clear_color(self.canvas_config.index, r, g, b, a));
        lib!(clear(self.canvas_config.index));
    }
    #[inline]
    pub fn root(&mut self) -> ForestNodeRc<Element> {
        self.root_node.clone()
    }
    #[inline]
    pub fn redraw(&mut self) {
        self.need_redraw = true;
    }
    #[inline]
    pub fn touching(&self) -> bool {
        self.touching
    }
    #[inline]
    pub fn touch_point(&self) -> element::Point {
        self.touch_point
    }
    #[inline]
    pub fn fetch_last_key_code(&mut self) -> KeyDescriptor {
        let last_key = self.last_key.clone();
        self.last_key = Default::default();
        last_key
    }

    fn generate_frame(&mut self) {
        let dirty = self.root_node.borrow().is_layout_dirty(); // any child or itself need update position offset
        if dirty || self.need_redraw {
            self.need_redraw = false;
            // let now = start_measure_time!();
            self.clear();
            let root_node_rc = self.root();
            let size = self.canvas_config.canvas_size.get();
            if dirty {
                root_node_rc.borrow_mut().dfs_update_position_offset(size);
            }
            root_node_rc.borrow_mut().draw(element::Position::new(0., 0., size.width(), size.height()), element::Transform::new());
            let rm = self.canvas_config.resource_manager();
            rm.borrow_mut().flush_draw();
            // debug!("Redraw time: {}ms", end_measure_time!(now));
        }
    }
}

const TOUCHSTART: i32 = 1;
const TOUCHMOVE: i32 = 2;
const TOUCHEND: i32 = 3;
const TOUCHCANCEL: i32 = 4;
const FREEMOVE: i32 = 5;
pub struct TouchEventDetail {
    pub client_x: f64,
    pub client_y: f64,
}
lib_define_callback! (TouchEventCallback (Rc<RefCell<CanvasContext>>) {
    fn callback(&mut self, touch_type: i32, x: i32, y: i32, _: i32) -> bool {
        let node = {
            let mut ctx = self.0.borrow_mut();
            match touch_type {
                TOUCHSTART => {
                    ctx.touching = true;
                    ctx.touch_point = element::Point::new(x as f64, y as f64);
                },
                TOUCHMOVE => {
                    ctx.touch_point = element::Point::new(x as f64, y as f64);
                },
                TOUCHEND => {
                    ctx.touch_point = element::Point::new(x as f64, y as f64);
                    ctx.touching = false;
                },
                TOUCHCANCEL => {
                    ctx.touch_point = element::Point::new(x as f64, y as f64);
                    ctx.touching = false;
                },
                FREEMOVE => {
                    ctx.touch_point = element::Point::new(x as f64, y as f64);
                },
                _ => {
                    panic!();
                }
            }
            ctx.root().borrow_mut().node_under_point(element::Point::new(x as f64, y as f64))
        };
        if node.is_some() {
            let event_name = String::from(match touch_type {
                TOUCHSTART => "touchstart",
                TOUCHMOVE => "touchmove",
                TOUCHEND => "touchend",
                TOUCHCANCEL => "touchcancel",
                _ => "",
            });
            node.unwrap().borrow_mut().dispatch_event(event_name, Box::new(TouchEventDetail {
                client_x: x as f64,
                client_y: y as f64,
            }), true);
        }
        true
    }
});

const KEY_DOWN: i32 = 1;
const KEY_PRESS: i32 = 2;
const KEY_UP: i32 = 3;
const SHIFT_KEY: i32 = 8;
const CTRL_KEY: i32 = 4;
const ALT_KEY: i32 = 2;
const LOGO_KEY: i32 = 1;
lib_define_callback! (KeyboardEventCallback (Rc<RefCell<CanvasContext>>) {
    fn callback(&mut self, event_type: i32, key_code: i32, _char_code: i32, special_keys: i32) -> bool {
        let mut ctx = self.0.borrow_mut();
        let kd = KeyDescriptor {
            key_code,
            is_down: if event_type == KEY_UP { false } else { true },
            shift: if special_keys & SHIFT_KEY > 0 { true } else { false },
            ctrl: if special_keys & CTRL_KEY > 0 { true } else { false },
            alt: if special_keys & ALT_KEY > 0 { true } else { false },
            logo: if special_keys & LOGO_KEY > 0 { true } else { false },
        };
        match event_type {
            KEY_DOWN => {
                ctx.last_key = kd;
            },
            KEY_PRESS => {
                ctx.last_key = kd;
            },
            KEY_UP => {
                ctx.last_key = kd;
            },
            _ => {
                panic!();
            }
        }
        true
    }
});

lib_define_callback! (CanvasSizeChangeCallback (Rc<RefCell<CanvasContext>>) {
    fn callback(&mut self, w: i32, h: i32, dpi: i32, _: i32) -> bool {
        let mut ctx = self.0.borrow_mut();
        ctx.set_canvas_size_inner(w, h, dpi as f64 / 100000000., false);
        true
    }
});
