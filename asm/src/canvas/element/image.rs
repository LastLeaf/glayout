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
    need_update: bool,
    loader: PretendSend<Option<Rc<RefCell<ImageLoader>>>>,
    natural_width: i32,
    natural_height: i32,
}

impl Image {
    pub fn new(cfg: &mut CanvasConfig) -> Self {
        Image {
            canvas_index: cfg.index,
            tex_id: cfg.alloc_tex_id(), // TODO change to dynamic
            need_update: true,
            loader: PretendSend::new(None),
            natural_width: 0,
            natural_height: 0,
        }
    }
    pub fn need_update_from_loader(&mut self) {
        // NOTE this method should be called if manually updated loader
        self.natural_width = 0;
        self.natural_height = 0;
        self.need_update = true;
    }
    pub fn set_loader(&mut self, loader: Rc<RefCell<ImageLoader>>) {
        self.need_update_from_loader();
        *self.loader = Some(loader);
    }
    pub fn load<T: Into<Vec<u8>>>(&mut self, url: T) {
        self.need_update_from_loader();
        if self.loader.is_none() {
            self.set_loader(Rc::new(RefCell::new(ImageLoader::new())))
        }
        ImageLoader::load(self.loader.as_mut().unwrap().clone(), url);
    }
    pub fn update_tex(&mut self) {
        self.need_update = false;
        lib!(tex_from_image(self.canvas_index, self.tex_id, self.loader.as_ref().unwrap().borrow().get_img_id()));
    }
}

impl super::ElementContent for Image {
    fn name(&self) -> &'static str {
        "Image"
    }
    fn draw(&mut self, style: &ElementStyle, bounding_rect: &BoundingRect) {
        if self.need_update {
            if self.loader.is_none() {
                return
            }
            {
                let loader = self.loader.as_ref().unwrap().borrow();
                if loader.get_status() != ImageLoaderStatus::Loaded {
                    return
                }
                let size = loader.get_size();
                self.natural_width = size.0;
                self.natural_height = size.1;
            }
            // NOTE for simplexity, tex generation is delayed to closest animation frame
            self.update_tex();
        }
        debug!("Attempted to draw an Image at ({}, {}) size ({}, {})", style.left, style.top, style.width, style.height);
        lib!(tex_draw(self.canvas_index, 0, self.tex_id, 0., 0., 1., 1., style.left, style.top, style.width, style.height));
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
    img_id: i32,
    width: i32,
    height: i32,
}

impl ImageLoader {
    pub fn new() -> Self {
        ImageLoader {
            status: ImageLoaderStatus::NotLoaded,
            img_id: CanvasConfig::alloc_image_id(),
            width: 0,
            height: 0,
        }
    }
    pub fn get_img_id(&self) -> i32 {
        self.img_id
    }
    pub fn get_status(&self) -> ImageLoaderStatus {
        self.status
    }
    pub fn get_size(&self) -> (i32, i32) {
        (self.width, self.height)
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
        loader.width = lib!(image_get_natural_width(loader.img_id));
        loader.height = lib!(image_get_natural_height(loader.img_id));
    }
});

impl Drop for ImageLoader {
    fn drop(&mut self) {
        lib!(image_unload(self.img_id));
    }
}
