use super::{Animation, TimingAnimation};

pub struct LinearTiming<T: TimingAnimation> {
    animation: T,
    start_value: f64,
    end_value: f64,
    current_value: f64
}

impl<T: TimingAnimation> LinearTiming<T> {
    pub fn new(animation: T, start_value: f64, end_value: f64) -> Self {
        return LinearTiming {
            animation,
            start_value,
            end_value,
            current_value: 0.
        };
    }
}

impl<T: TimingAnimation> Animation for LinearTiming<T> {
    fn frame(&mut self, _current_frame: i32, _total_frames: i32, current_time: f64, total_time: f64) {
        self.current_value = current_time / total_time * (self.end_value - self.start_value) + self.start_value;
        self.animation.progress(self.current_value, current_time, total_time);
    }
    fn end(&mut self, _total_frames: i32, total_time: f64) {
        self.current_value = self.end_value;
        self.animation.progress(self.end_value, total_time, total_time);
    }
}
