use glayout::frame::animation::{Animation, TimingAnimation};
use std::time;

fn ani_time_to_ms(dur: time::Duration) -> f64 {
    dur.as_secs() as f64 * 1000. + dur.subsec_nanos() as f64 / 1_000_000.
}

pub fn init() {
    register_test_case!(module_path!(), _ctx, {
        TimingAnimation::new(Box::new(move |current_time, total_time| -> bool {
            debug!("Animation progress: {:?}", ani_time_to_ms(current_time) / ani_time_to_ms(total_time));
            true
        }), time::Duration::new(3, 0), false).exec();
        return 0;
    });
}
