#![macro_use]

use std::rc::Rc;
use std::cell::{RefCell};
use std::time;
use super::utils::PretendSend;

pub enum FramePriority {
    High,
    Normal,
    Low,
}

lazy_static! {
    static ref FRAME_OBJECTS: PretendSend<Rc<RefCell<FrameObjectsGroup>>> = PretendSend::new(Rc::new(RefCell::new(FrameObjectsGroup::new())));
}

pub type Frame = FnMut(time::Instant) -> bool + 'static;

#[derive(Clone)]
pub struct FrameCallback {
    f: Rc<RefCell<Box<Frame>>>,
}

impl FrameCallback {
    pub fn new(f: Box<Frame>) -> Self {
        Self {
            f: Rc::new(RefCell::new(f))
        }
    }
}

#[derive(Clone)]
struct FrameObjectsGroup {
    count: u32,
    high: Vec<FrameCallback>,
    normal: Vec<FrameCallback>,
    low: Vec<FrameCallback>,
}

impl FrameObjectsGroup {
    fn new() -> Self {
        Self {
            count: 0,
            high: Vec::new(),
            normal: Vec::new(),
            low: Vec::new(),
        }
    }
    #[inline]
    fn get(&mut self, priority: FramePriority) -> &mut Vec<FrameCallback> {
        match priority {
            FramePriority::High => &mut self.high,
            FramePriority::Normal => &mut self.normal,
            FramePriority::Low => &mut self.low,
        }
    }
}

pub fn bind(f: FrameCallback, priority: FramePriority) {
    let mut fog = FRAME_OBJECTS.borrow_mut();
    if fog.count == 0 {
        lib!(enable_animation_frame());
    }
    fog.count += 1;
    let frame_objects = fog.get(priority);
    frame_objects.push(f);
}

pub fn unbind(f: FrameCallback, priority: FramePriority) -> bool {
    let mut fog = FRAME_OBJECTS.borrow_mut();
    let ret = {
        let frame_objects = fog.get(priority);
        let ret = match frame_objects.iter().position(|x| Rc::ptr_eq(&x.f, &f.f)) {
            None => false,
            Some(index) => {
                frame_objects.remove(index);
                true
            }
        };
        ret
    };
    if ret {
        fog.count -= 1;
        if fog.count == 0 {
            lib!(disable_animation_frame());
        }
    }
    ret
}

#[macro_export]
macro_rules! frame {
    ($f:expr) => {
        $crate::frame::bind($crate::frame::FrameCallback::new(Box::new($f)), $crate::frame::FramePriority::Normal);
    }
}

macro_rules! exec {
    ($x: expr, $y: expr) => {
        |x: &mut FrameCallback| {
            let f = &mut *x.f.borrow_mut();
            let ret = f($y);
            if ret == false {
                unbind(x.clone(), $x);
            }
        }
    }
}

pub fn generate(timestamp: time::Instant) {
    let mut fo = (*FRAME_OBJECTS.borrow_mut()).clone();
    fo.high.iter_mut().for_each(exec!(FramePriority::High, timestamp));
    fo.normal.iter_mut().for_each(exec!(FramePriority::Normal, timestamp));
    fo.low.iter_mut().for_each(exec!(FramePriority::Low, timestamp));
}

pub mod animation;
