use std::time;

pub type TimingAnimationCallback = FnMut(time::Duration, time::Duration) -> bool + 'static;
pub type FrameAnimationCallback = FnMut(i32, i32) -> bool + 'static;

pub trait Animation: Sized {
    fn exec(self) {}
}

pub struct TimingAnimation {
    total_time: time::Duration,
    loop_animation: bool,
    f: Box<TimingAnimationCallback>,
}

pub struct FrameAnimation {
    total_frames: i32,
    loop_animation: bool,
    f: Box<FrameAnimationCallback>,
}

impl TimingAnimation {
    pub fn new(f: Box<TimingAnimationCallback>, total_time: time::Duration, loop_animation: bool) -> Self {
        Self {
            total_time,
            loop_animation,
            f,
        }
    }
}

impl Animation for TimingAnimation {
    fn exec(mut self) {
        let mut start_time = time::Instant::now();
        frame!(move |t| {
            let mut dur = t - start_time;
            let mut cont = true;
            if dur > self.total_time {
                dur = match self.loop_animation {
                    true => {
                        while dur > self.total_time {
                            dur -= self.total_time;
                            start_time += self.total_time;
                        }
                        dur
                    },
                    false => {
                        cont = false;
                        self.total_time
                    },
                };
            }
            let f = &mut *self.f;
            let ret = f(dur, self.total_time);
            if !ret { return false };
            return cont;
        });
    }
}

impl FrameAnimation {
    pub fn new(f: Box<FrameAnimationCallback>, total_frames: i32, loop_animation: bool) -> Self {
        Self {
            total_frames,
            loop_animation,
            f,
        }
    }
}

impl Animation for FrameAnimation {
    fn exec(mut self) {
        let mut current_frame = -1;
        frame!(move |_t| {
            current_frame += 1;
            let mut cont = true;
            if current_frame == self.total_frames {
                current_frame = match self.loop_animation {
                    true => 0,
                    false => {
                        cont = false;
                        current_frame
                    },
                };
            }
            let f = &mut *self.f;
            let ret = f(current_frame, self.total_frames);
            if !ret { return false };
            return cont;
        });
    }
}
