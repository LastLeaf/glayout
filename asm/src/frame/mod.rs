pub mod animation;

use std::rc::Rc;
use std::cell::RefCell;
use super::utils::PretendSend;

pub trait Frame {
    fn frame(&mut self, timestamp: f64) -> bool;
}

lazy_static! {
    static ref FRAME_OBJECTS: PretendSend<Rc<RefCell<Vec<Rc<RefCell<Frame>>>>>> = PretendSend::new(Rc::new(RefCell::new(vec![])));
}

pub fn bind(fo: Rc<RefCell<Frame>>) {
    let mut frame_objects = FRAME_OBJECTS.borrow_mut();
    if frame_objects.len() == 0 {
        lib!(enable_animation_frame());
    }
    frame_objects.push(fo);
}

pub fn unbind(fo: Rc<RefCell<Frame>>) -> bool {
    let mut frame_objects = FRAME_OBJECTS.borrow_mut();
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

pub fn generate(timestamp: f64) {
    FRAME_OBJECTS.borrow_mut().iter_mut().for_each(|x| {
        let ret = x.borrow_mut().frame(timestamp);
        if ret == false {
            unbind(x.clone());
        }
    });
}
