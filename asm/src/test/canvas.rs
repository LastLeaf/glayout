use std::rc::Rc;
use std::cell::RefCell;
use glayout::canvas::Canvas;
use glayout::frame::animation::{TimingAnimation, AnimationObject, LinearTiming};

pub fn init() {
    register_test_case!(module_path!(), {
        let mut canvas = Canvas::new(0);

        canvas.ctx(|ctx| {
            ctx.set_canvas_size(400, 300, 1.);
        });

        struct BackgroundColorAni(Canvas);
        impl TimingAnimation for BackgroundColorAni {
            fn progress(&mut self, current_value: f64, _current_time: f64, _total_time: f64) {
                self.0.ctx(|ctx| {
                    ctx.set_clear_color(0., current_value as f32, current_value as f32, 1.);
                })
            }
        }

        let ani_obj = Rc::new(RefCell::new(AnimationObject::new(Box::new(LinearTiming::new(BackgroundColorAni(canvas.clone()), 0., 1.)))));
        AnimationObject::exec(ani_obj, 0, 3000.);

        return 0;
    });
}
