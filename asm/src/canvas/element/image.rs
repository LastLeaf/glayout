use std::ffi::CString;
use std::rc::Rc;
use std::cell::RefCell;
use super::super::super::utils::PretendSend;
use super::super::CanvasConfig;
use super::{ElementStyle, BoundingRect};

// basic image element

pub struct Image {
    canvas_index: i32,
    tex_id: i32,
    waiting_loader: bool,
    loader: PretendSend<Option<Rc<RefCell<ImageLoader>>>>
}

impl Image {
    pub fn new(cfg: &mut CanvasConfig) -> Self {
        Image {
            canvas_index: cfg.index,
            tex_id: cfg.alloc_tex_id(), // TODO change to dynamic
            waiting_loader: false,
            loader: PretendSend::new(None)
        }
    }
    pub fn set_loader(&mut self, loader: Rc<RefCell<ImageLoader>>) {
        self.waiting_loader = true;
        *self.loader = Some(loader);
    }
    pub fn load<T: Into<Vec<u8>>>(&mut self, url: T) {
        self.waiting_loader = true;
        if self.loader.is_none() {
            self.set_loader(Rc::new(RefCell::new(ImageLoader::new())))
        }
        ImageLoader::load(self.loader.as_mut().unwrap().clone(), url);
    }
    pub fn update_tex(&mut self) {
        self.waiting_loader = false;
        lib!(tex_from_image(self.canvas_index, self.tex_id, self.loader.as_ref().unwrap().borrow().get_img_id()));
    }
}

impl super::ElementContent for Image {
    fn name(&self) -> &'static str {
        "Image"
    }
    fn draw(&mut self, style: &ElementStyle, bounding_rect: &BoundingRect) {
        if self.loader.is_none() || self.loader.as_ref().unwrap().borrow().get_status() != ImageLoaderStatus::Loaded {
            return
        }
        if self.waiting_loader {
            self.update_tex()
        }
        debug!("Attempted to draw an Image at ({}, {}) size ({}, {})", style.left, style.top, style.width, style.height);
        lib!(tex_draw(self.canvas_index, 0, self.tex_id, style.left, style.top, style.width, style.height, style.left, style.top, style.width, style.height));
        lib!(tex_draw_end(self.canvas_index, 1));
    }
}

// image loader

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImageLoaderStatus {
    NotLoaded,
    Loading,
    Loaded,
}

pub struct ImageLoader {
    status: ImageLoaderStatus,
    img_id: i32
}

impl ImageLoader {
    pub fn new() -> Self {
        ImageLoader {
            status: ImageLoaderStatus::NotLoaded,
            img_id: CanvasConfig::alloc_image_id(),
        }
    }
    pub fn get_img_id(&self) -> i32 {
        self.img_id
    }
    pub fn get_status(&self) -> ImageLoaderStatus {
        self.status
    }
    pub fn load<T: Into<Vec<u8>>>(self_rc: Rc<RefCell<Self>>, url: T) {
        let mut self_ref = self_rc.borrow_mut();
        lib!(image_unload(self_ref.img_id));
        self_ref.status = ImageLoaderStatus::Loading;
        lib!(image_load_url(self_ref.img_id, CString::new(url).unwrap().into_raw(), lib_callback!(ImageLoaderCallback(self_rc.clone()))));
    }
}

struct ImageLoaderCallback (Rc<RefCell<ImageLoader>>);

lib_define_callback! (ImageLoaderCallback {
    fn callback(&mut self, _ret_code: i32) {
        let mut loader = self.0.borrow_mut();
        loader.status = ImageLoaderStatus::Loaded;
    }
});

impl Drop for ImageLoader {
    fn drop(&mut self) {
        lib!(image_unload(self.img_id));
    }
}
