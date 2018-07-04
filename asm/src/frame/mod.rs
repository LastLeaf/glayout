pub mod animation;

use std::rc::Rc;
use std::cell::{RefCell};
use super::utils::PretendSend;

pub enum FramePriority {
    High,
    Normal,
    Low,
}

pub trait Frame {
    fn frame(&mut self, timestamp: f64) -> bool;
}

lazy_static! {
    static ref FRAME_OBJECTS: PretendSend<Rc<RefCell<FrameObjectsGroup>>> = PretendSend::new(Rc::new(RefCell::new(FrameObjectsGroup::new())));
}

struct FrameObjectsGroup {
    high: Vec<Rc<RefCell<Frame>>>,
    normal: Vec<Rc<RefCell<Frame>>>,
    low: Vec<Rc<RefCell<Frame>>>,
}

impl FrameObjectsGroup {
    fn new() -> Self {
        Self {
            high: vec![],
            normal: vec![],
            low: vec![],
        }
    }
    #[inline]
    fn get(&mut self, priority: FramePriority) -> &mut Vec<Rc<RefCell<Frame>>> {
        match priority {
            FramePriority::High => &mut self.high,
            FramePriority::Normal => &mut self.normal,
            FramePriority::Low => &mut self.low,
        }
    }
}

pub fn bind(fo: Rc<RefCell<Frame>>, priority: FramePriority) {
    let mut fog = FRAME_OBJECTS.borrow_mut();
    let frame_objects = fog.get(priority);
    if frame_objects.len() == 0 {
        lib!(enable_animation_frame());
    }
    frame_objects.push(fo);
}

pub fn unbind(fo: Rc<RefCell<Frame>>, priority: FramePriority) -> bool {
    let mut fog = FRAME_OBJECTS.borrow_mut();
    let frame_objects = fog.get(priority);
    return match frame_objects.iter().position(|ref x| Rc::ptr_eq(&x, &fo)) {
        None => false,
        Some(index) => {
            frame_objects.remove(index);
            if frame_objects.len() == 0 {
                lib!(disable_animation_frame());
            }
            return true;
        }
    };
}

macro_rules! exec {
    ($x: expr, $y: expr) => {
        |x| {
            let ret = x.borrow_mut().frame($y);
            if ret == false {
                unbind(x.clone(), $x);
            }
        }
    }
}

pub fn generate(timestamp: f64) {
    FRAME_OBJECTS.borrow_mut().high.iter_mut().for_each(exec!(FramePriority::High, timestamp));
    FRAME_OBJECTS.borrow_mut().normal.iter_mut().for_each(exec!(FramePriority::Normal, timestamp));
    FRAME_OBJECTS.borrow_mut().low.iter_mut().for_each(exec!(FramePriority::Low, timestamp));
}
