use std::rc::Rc;
use std::cell::RefCell;
use glayout::frame;
use glayout::canvas::{Canvas, CanvasContext};

struct MainLoop {
    _canvas: Canvas,
    ctx: Rc<RefCell<CanvasContext>>,
}

impl frame::Frame for MainLoop {
    fn frame(&mut self, _timestamp: f64) -> bool {
        let ctx = self.ctx.borrow_mut();

        if ctx.touching() {
            println!("Touching: {:?}", ctx.touch_point());
        }

        return true;
    }
}

pub fn init() {
    register_test_case!(module_path!(), {
        let mut canvas = Canvas::new(0);
        let ctx = canvas.context().clone();

        canvas.ctx(|ctx| {
            let pixel_ratio = ctx.device_pixel_ratio();
            ctx.set_canvas_size(800, 600, pixel_ratio);
            ctx.set_clear_color(0.5, 1., 0.5, 1.);
        });

        frame::bind(Rc::new(RefCell::new(MainLoop {
            _canvas: canvas,
            ctx,
        })), frame::FramePriority::Normal);

        return 0;
    });
}
