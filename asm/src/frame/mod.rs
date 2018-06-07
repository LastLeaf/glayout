pub mod animation;

use std::sync::{Arc, Mutex};

pub trait Frame: Send {
    fn frame(&mut self, timestamp: f64) -> bool;
}

lazy_static! {
    static ref FRAME_OBJECTS: Arc<Mutex<Vec<Arc<Mutex<Frame>>>>> = Arc::new(Mutex::new(vec![]));
}

pub fn bind(fo: Arc<Mutex<Frame>>) {
    let mut frame_objects = FRAME_OBJECTS.lock().unwrap();
    if frame_objects.len() == 0 {
        lib!(enable_animation_frame());
    }
    frame_objects.push(fo);
}

pub fn unbind(fo: Arc<Mutex<Frame>>) -> bool {
    let mut frame_objects = FRAME_OBJECTS.lock().unwrap();
    return match frame_objects.iter().position(|ref x| Arc::ptr_eq(&x, &fo)) {
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
    FRAME_OBJECTS.lock().unwrap().iter_mut().for_each(|x| {
        let ret = x.lock().unwrap().frame(timestamp);
        if ret == false {
            unbind(x.clone());
        }
    });
}
