use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::os::raw::c_char;
use std::ffi::CStr;
use std::path::Path;
use std::thread;
use std::time::Instant;
use image;
use super::layout_thread;
use super::super::Callback;
use super::super::super::utils::PretendSend;

lazy_static! {
    static ref IMAGES: Arc<Mutex<HashMap<i32, (i32, i32, Box<[u8]>)>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub fn image_load_url(id: i32, url: *mut c_char, cb_ptr: *mut Box<Callback>) {
    let url = unsafe { CStr::from_ptr(url) };
    let url = Path::new(url.to_str().unwrap());
    let cb_ptr = PretendSend::new(cb_ptr);
    thread::spawn(move || {
        let rgba_image = image::open(url).unwrap().to_rgba();
        let image_info = (rgba_image.width() as i32, rgba_image.height() as i32, rgba_image.into_raw().into_boxed_slice());
        IMAGES.lock().unwrap().insert(id, image_info);
        layout_thread::push_event(Instant::now(), layout_thread::EventDetail::ImageLoadEvent, move |_time, _detail| {
            super::super::callback(*cb_ptr, 0, 0, 0, 0);
        })
    });
}
pub fn image_unload(id: i32) {
    IMAGES.lock().unwrap().remove(&id);
}
pub fn image_get_natural_width(id: i32) -> i32 {
    IMAGES.lock().unwrap().get(&id).unwrap().0 as i32
}
pub fn image_get_natural_height(id: i32) -> i32 {
    IMAGES.lock().unwrap().get(&id).unwrap().1 as i32
}
pub fn tex_from_image(canvas_index: i32, tex_id: i32, img_id: i32) {
    let images = &IMAGES.lock().unwrap();
    let image = &images[&img_id];
    let original = &*image.2;
    let mut premultiplied: Vec<u8> = Vec::with_capacity(original.len());
	for i in 0..(original.len() / 4) {
        let p = i * 4;
        let a = original[p + 3] as f32 / 255.;
		premultiplied.extend_from_slice(&[
            (original[p] as f32 * a).floor() as u8,
            (original[p + 1] as f32 * a).floor() as u8,
            (original[p + 2] as f32 * a).floor() as u8,
            original[p + 3]
        ]);
	}
    super::tex_manager::tex_create(canvas_index, image.0, image.1, premultiplied, tex_id);
}
