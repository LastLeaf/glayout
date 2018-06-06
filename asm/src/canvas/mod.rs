use std::sync::{Arc, Mutex};
use super::frame;
use super::tree::{TreeNodeRc, TreeNodeSearchType};

mod element;
mod config;

pub type CanvasConfig = config::CanvasConfig;
pub type Element = element::Element;
pub type EmptyElement = element::EmptyElement;

pub struct CanvasContext {
    canvas_config: CanvasConfig,
    root_element: TreeNodeRc<Element>,
}

// TODO use a new way to do this
unsafe impl Send for CanvasContext { }
unsafe impl Sync for CanvasContext { }

#[derive(Clone)]
pub struct Canvas {
    context: Arc<Mutex<CanvasContext>>
}

impl Canvas {
    pub fn new(index: i32) -> Self {
        lib!(bind_canvas(index));
        let mut canvas_config = CanvasConfig {
            index,
            tex_size: lib!(tex_get_size(index)) as i32,
            tex_count: lib!(tex_get_count(index)) as i32,
            tex_max_draws: lib!(tex_get_max_draws()) as i32,
            image_id_inc: 1,
        };
        println!("Canvas binded: tex_size {}; tex_count {}; tex_max_draws {}", canvas_config.tex_size, canvas_config.tex_count, canvas_config.tex_max_draws);
        let root_element = element! {
            [&mut canvas_config] EmptyElement
        };
        let arc_ctx = Arc::new(Mutex::new(CanvasContext {
            canvas_config,
            root_element,
        }));
        frame::bind(arc_ctx.clone());
        return Canvas {
            context: arc_ctx
        };
    }
    pub fn destroy(&mut self) {
        frame::unbind(self.context.clone());
    }
    pub fn get_context(&self) -> Arc<Mutex<CanvasContext>> {
        self.context.clone()
    }
    pub fn context<F>(&mut self, f: F) where F: Fn(&mut CanvasContext) {
        f(&mut *self.context.lock().unwrap());
    }
}

impl Drop for CanvasContext {
    fn drop(&mut self) {
        lib!(unbind_canvas(self.canvas_config.index));
    }
}

impl frame::Frame for CanvasContext {
    fn frame(&mut self, _timestamp: f64) -> bool {
        self.clear();
        let mut root_element_rc = self.get_root();
        root_element_rc.dfs(TreeNodeSearchType::ChildrenLast, &|element: &mut Element| {
            element.draw();
        });
        return true;
    }
}

impl CanvasContext {
    pub fn get_canvas_config(&mut self) -> &mut CanvasConfig {
        &mut self.canvas_config
    }
    pub fn set_canvas_size(&mut self, w: i32, h: i32) {
        lib!(set_canvas_size(self.canvas_config.index, w, h));
    }
    pub fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        lib!(set_clear_color(self.canvas_config.index, r, g, b, a));
    }
    pub fn clear(&mut self) {
        lib!(clear(self.canvas_config.index));
    }
    pub fn get_root(&mut self) -> TreeNodeRc<Element> {
        self.root_element.clone()
    }
}

pub mod test {
    use std::sync::{Arc, Mutex};
    use super::Canvas;
    use super::super::frame::animation::{TimingAnimation, AnimationObject, LinearTiming};

    pub fn test() -> i32 {
        let mut canvas = Canvas::new(0);

        canvas.context(|ctx| {
            ctx.set_canvas_size(400, 300);
        });

        struct BackgroundColorAni(Canvas);
        impl TimingAnimation for BackgroundColorAni {
            fn progress(&mut self, current_value: f64, _current_time: f64, _total_time: f64) {
                self.0.context(|ctx| {
                    ctx.set_clear_color(0., current_value as f32, current_value as f32, 1.);
                })
            }
        }

        let ani_obj = Arc::new(Mutex::new(AnimationObject::new(Box::new(LinearTiming::new(BackgroundColorAni(canvas.clone()), 0., 1.)))));
        AnimationObject::exec(ani_obj, 0, 3000.);

        let mut err = 0;
        err += super::element::test::test();
        return err;
    }
}
