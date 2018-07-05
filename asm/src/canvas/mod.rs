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
pub type EmptyElement = element::EmptyElement;

pub struct CanvasContext {
    canvas_config: Rc<CanvasConfig>,
    root_element: TreeNodeRc<Element>,
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
            lib!(get_device_pixel_ratio()) as f64
        ));
        log!("Canvas binded: tex_size {}; tex_count {}; tex_max_draws {}", canvas_config.tex_size, canvas_config.tex_count, canvas_config.tex_max_draws);
        let root_element = element! {
            [&canvas_config] EmptyElement
        };
        let ctx = Rc::new(RefCell::new(CanvasContext {
            canvas_config,
            root_element,
        }));
        frame::bind(ctx.clone(), frame::FramePriority::Low);
        return Canvas {
            context: ctx
        };
    }
    pub fn destroy(&mut self) {
        frame::unbind(self.context.clone(), frame::FramePriority::Low);
    }
    pub fn get_context(&self) -> Rc<RefCell<CanvasContext>> {
        self.context.clone()
    }
    pub fn context<F>(&mut self, f: F) where F: Fn(&mut CanvasContext) {
        f(&mut *self.context.borrow_mut());
    }
}

impl Drop for CanvasContext {
    fn drop(&mut self) {
        lib!(unbind_canvas(self.canvas_config.index));
    }
}

impl frame::Frame for CanvasContext {
    fn frame(&mut self, _timestamp: f64) -> bool {
        let dirty = self.canvas_config.clear_dirty();
        if dirty {
            let now = start_measure_time!();
            self.clear();
            let mut root_element_rc = self.get_root();
            root_element_rc.dfs(TreeNodeSearchType::ChildrenLast, &mut |node| {
                node.get_mut().draw();
                true
            });
            let rm = self.canvas_config.get_resource_manager();
            rm.borrow_mut().flush_draw();
            debug!("Redraw time: {}ms", end_measure_time!(now));
        }
        return true;
    }
}

impl CanvasContext {
    pub fn get_canvas_config(&mut self) -> Rc<CanvasConfig> {
        self.canvas_config.clone()
    }
    pub fn set_canvas_size(&mut self, w: i32, h: i32, pixel_ratio: f64) {
        lib!(set_canvas_size(self.canvas_config.index, w, h, pixel_ratio));
    }
    pub fn get_device_pixel_ratio(&self) -> f64 {
        lib!(get_device_pixel_ratio())
    }
    pub fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.canvas_config.set_clear_color((r, g, b, a));
        lib!(set_clear_color(self.canvas_config.index, r, g, b, a));
    }
    pub fn clear(&mut self) {
        let (r, g, b, a) = self.canvas_config.get_clear_color();
        lib!(set_clear_color(self.canvas_config.index, r, g, b, a));
        lib!(clear(self.canvas_config.index));
    }
    pub fn get_root(&mut self) -> TreeNodeRc<Element> {
        self.root_element.clone()
    }
    pub fn get_node_by_id(&mut self, id: &'static str) -> Option<TreeNodeRc<Element>> {
        let mut ret = None;
        self.root_element.dfs(TreeNodeSearchType::ChildrenLast, &mut |node| {
            if node.get_mut().style.id == id {
                ret = Some(node.clone());
                return false;
            }
            true
        });
        ret
    }
}
