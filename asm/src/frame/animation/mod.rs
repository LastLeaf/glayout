mod linear_timing;

use std::sync::{Arc, Mutex};

pub type LinearTiming<T> = linear_timing::LinearTiming<T>;

pub trait Animation: Send {
    fn frame(&mut self, current_frame: i32, total_frames: i32, current_time: f64, total_time: f64);
    fn end(&mut self, total_frames: i32, total_time: f64);
}

pub trait TimingAnimation: Send {
    fn progress(&mut self, current_value: f64, current_time: f64, total_time: f64);
}

pub struct AnimationObject {
    start_time: f64,
    total_time: f64,
    current_frame: i32,
    total_frames: i32,
    animation: Box<Animation>
}

impl super::Frame for AnimationObject {
    fn frame(&mut self, timestamp: f64) -> bool {
        if self.total_time <= timestamp - self.start_time && self.current_frame >= self.total_frames {
            self.animation.end(self.total_frames, self.total_time);
            return false;
        }
        self.animation.frame(self.current_frame, self.total_frames, timestamp - self.start_time, self.total_time);
        self.current_frame += 1;
        return true;
    }
}

impl AnimationObject {
    pub fn new(ani: Box<Animation>) -> Self {
        AnimationObject {
            start_time: 0.,
            total_time: 0.,
            current_frame: 0,
            total_frames: 0,
            animation: ani,
        }
    }
    pub fn exec(arc_ani: Arc<Mutex<AnimationObject>>, total_frames: i32, total_time: f64) {
        {
            let mut ani = arc_ani.lock().unwrap();
            ani.total_frames = total_frames;
            ani.total_time = total_time;
        }
        super::bind(arc_ani);
    }
}

pub mod test {
    pub fn test() -> i32 {
        let mut err = 0;
        err += super::linear_timing::test::test();
        return err;
    }
}
