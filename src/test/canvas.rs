use std::time;
use glayout::frame::animation::{Animation, TimingAnimation};

fn ani_time_to_ms(dur: time::Duration) -> f64 {
    dur.as_secs() as f64 * 1000. + dur.subsec_nanos() as f64 / 1_000_000.
}

pub fn init() {
    register_test_case!(module_path!(), rc_context, {
        let mut context = rc_context.borrow_mut();
        context.set_canvas_size(256, 256, 1.);

        let rc_context = rc_context.clone();
        TimingAnimation::new(Box::new(move |cur, total| {
            let ratio = ani_time_to_ms(cur) / ani_time_to_ms(total);
            let mut context = rc_context.borrow_mut();
            context.set_clear_color(0., ratio as f32, ratio as f32, 1.);
            context.redraw();
            true
        }), time::Duration::new(3, 0), false).exec();

        return 0;
    });
}
