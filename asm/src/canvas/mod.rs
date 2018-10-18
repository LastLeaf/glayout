use std::rc::Rc;
use std::cell::RefCell;
use super::frame;
use super::tree::{TreeNodeRc, TreeNodeSearchType};

pub mod element;
mod config;
mod character;
mod resource;

pub type CanvasConfig = config::CanvasConfig;
pub type Element = element::Element;
pub type Empty = element::Empty;

#[derive(Default, Clone, Debug)]
pub struct KeyDescriptor {
    key_code: i32,
    shift: bool,
    ctrl: bool,
    alt: bool,
    logo: bool,
}

pub struct CanvasContext {
    canvas_config: Rc<CanvasConfig>,
    root_node: TreeNodeRc<Element>,
    need_redraw: bool,
    touching: bool,
    touch_point: (f64, f64),
    last_key: KeyDescriptor,
}

#[derive(Clone)]
pub struct Canvas {
    context: Rc<RefCell<CanvasContext>>
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
        let root_node = element! {
            [&canvas_config] Empty
        };
        let ctx = Rc::new(RefCell::new(CanvasContext {
            canvas_config,
            root_node,
            need_redraw: false,
            touching: false,
            touch_point: (0., 0.),
            last_key: Default::default(),
        }));
        frame::bind(ctx.clone(), frame::FramePriority::Low);
        lib!(bind_touch_events(index, lib_callback!(TouchEventCallback(ctx.clone()))));
        lib!(bind_keyboard_events(index, lib_callback!(KeyboardEventCallback(ctx.clone()))));
        lib!(bind_canvas_size_change(index, lib_callback!(CanvasSizeChangeCallback(ctx.clone()))));
        return Canvas {
            context: ctx
        };
    }
    #[inline]
    pub fn destroy(&mut self) {
        frame::unbind(self.context.clone(), frame::FramePriority::Low);
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

impl frame::Frame for CanvasContext {
    fn frame(&mut self, _timestamp: f64) -> bool {
        let dirty = self.root_node.elem().is_dirty(); // any child or itself need update position offset
        if dirty || self.need_redraw {
            self.need_redraw = false;
            // let now = start_measure_time!();
            self.clear();
            let root_node_rc = self.root();
            let size = self.canvas_config.canvas_size.get();
            if dirty {
                root_node_rc.elem().dfs_update_position_offset(size);
            }
            root_node_rc.elem().draw((0., 0., size.0, size.1), element::Transform::new());
            let rm = self.canvas_config.resource_manager();
            rm.borrow_mut().flush_draw();
            // debug!("Redraw time: {}ms", end_measure_time!(now));
        }
        return true;
    }
}

impl CanvasContext {
    #[inline]
    pub fn canvas_config(&mut self) -> Rc<CanvasConfig> {
        self.canvas_config.clone()
    }
    fn set_canvas_size_inner(&mut self, w: i32, h: i32, pixel_ratio: f64, update_logical_size: bool) {
        log!("Canvas size changed: {}", self.canvas_config.index);
        self.canvas_config.canvas_size.set((w as f64, h as f64));
        lib!(set_canvas_size(self.canvas_config.index, w, h, pixel_ratio, update_logical_size as i32));
        self.root_node.elem().mark_dirty();
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
    pub fn root(&mut self) -> TreeNodeRc<Element> {
        self.root_node.clone()
    }
    pub fn node_by_id(&mut self, id: &'static str) -> Option<TreeNodeRc<Element>> {
        let mut ret = None;
        self.root_node.dfs(TreeNodeSearchType::ChildrenLast, &mut |node| {
            if node.elem().style().get_id() == id {
                ret = Some(node.clone());
                return false;
            }
            true
        });
        ret
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
    pub fn touch_point(&self) -> (f64, f64) {
        self.touch_point
    }
    #[inline]
    pub fn fetch_last_key_code(&mut self) -> KeyDescriptor {
        let last_key = self.last_key.clone();
        self.last_key = Default::default();
        last_key
    }
}

const TOUCHSTART: i32 = 1;
const TOUCHMOVE: i32 = 2;
const TOUCHEND: i32 = 3;
const FREEMOVE: i32 = 4;
lib_define_callback! (TouchEventCallback (Rc<RefCell<CanvasContext>>) {
    fn callback(&mut self, touch_type: i32, x: i32, y: i32, _: i32) -> bool {
        let mut ctx = self.0.borrow_mut();
        match touch_type {
            TOUCHSTART => {
                ctx.touching = true;
                ctx.touch_point = (x as f64, y as f64);
            },
            TOUCHMOVE => {
                ctx.touch_point = (x as f64, y as f64);
            },
            TOUCHEND => {
                ctx.touch_point = (x as f64, y as f64);
                ctx.touching = false;
            },
            FREEMOVE => {
                ctx.touch_point = (x as f64, y as f64);
            },
            _ => {
                panic!();
            }
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
            shift: if special_keys & SHIFT_KEY > 0 { true } else { false },
            ctrl: if special_keys & CTRL_KEY > 0 { true } else { false },
            alt: if special_keys & ALT_KEY > 0 { true } else { false },
            logo: if special_keys & LOGO_KEY > 0 { true } else { false },
        };
        match event_type {
            KEY_DOWN => {
                // TODO
            },
            KEY_PRESS => {
                // TODO
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
