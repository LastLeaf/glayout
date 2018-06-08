use std::sync::{Arc, Mutex};
use glayout::frame::animation::{AnimationObject, TimingAnimation};
use glayout::frame::animation::{LinearTiming};

pub struct TestAnimation();
impl TimingAnimation for TestAnimation {
    fn progress(&mut self, _current_value: f64, _current_time: f64, _total_time: f64) {
        debug!("Animation progress: {}", _current_value);
    }
}

pub fn init() {
    register_test_case!(module_path!(), {
        let ani_obj = Arc::new(Mutex::new(AnimationObject::new(Box::new(LinearTiming::new(TestAnimation(), 0., 100.)))));
        AnimationObject::exec(ani_obj, 0, 3000.);
        return 0;
    });
}
