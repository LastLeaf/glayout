use std::ffi::CString;
use std::rc::Rc;
use std::cell::RefCell;
use super::super::CanvasConfig;
use super::super::resource::ResourceManager;
use super::{ElementStyle, BoundingRect};

const IMAGE_SIZE_WARN: i32 = 4096;

// basic image element

pub struct Image {
    canvas_config: Rc<CanvasConfig>,
    tex_id: i32,
    need_update: bool,
    loader: Option<Rc<RefCell<ImageLoader>>>,
    natural_width: i32,
    natural_height: i32,
}

impl Image {
    pub fn new(cfg: &Rc<CanvasConfig>) -> Self {
        Image {
            canvas_config: cfg.clone(),
            tex_id: -1,
            need_update: false,
            loader: None,
            natural_width: 0,
            natural_height: 0,
        }
    }
    pub fn need_update_from_loader(&mut self) {
        // NOTE this method should be called if manually updated loader
        self.natural_width = 0;
        self.natural_height = 0;
        self.need_update = true;
        self.canvas_config.mark_dirty();
    }
    pub fn set_loader(&mut self, loader: Rc<RefCell<ImageLoader>>) {
        self.need_update_from_loader();
        self.loader = Some(loader);
    }
    pub fn load<T: Into<Vec<u8>>>(&mut self, url: T) {
        self.need_update_from_loader();
        let cc = self.canvas_config.clone();
        self.set_loader(Rc::new(RefCell::new(ImageLoader::new_with_canvas_config(cc))));
        ImageLoader::load(self.loader.as_mut().unwrap().clone(), url);
    }
    pub fn update_tex(&mut self) {
        self.need_update = false;
        let loader = self.loader.as_ref().unwrap();
        self.tex_id = loader.borrow().tex_id;
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
        // debug!("Attempted to draw an Image at ({}, {}) size ({}, {})", style.left, style.top, style.width, style.height);
        let rm = self.canvas_config.get_resource_manager();
        rm.borrow_mut().request_draw(
            self.tex_id,
            0., 0., 1., 1.,
            style.left, style.top, style.width, style.height
        );
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
    canvas_config: Rc<CanvasConfig>,
    status: ImageLoaderStatus,
    img_id: i32,
    tex_id: i32,
    width: i32,
    height: i32,
}

impl ImageLoader {
    pub fn new_with_canvas_config(cfg: Rc<CanvasConfig>) -> Self {
        ImageLoader {
            canvas_config: cfg,
            status: ImageLoaderStatus::NotLoaded,
            img_id: ResourceManager::alloc_image_id(),
            tex_id: -1,
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
        assert_eq!(self_ref.status, ImageLoaderStatus::NotLoaded);
        self_ref.status = ImageLoaderStatus::Loading;
        lib!(image_load_url(self_ref.img_id, CString::new(url).unwrap().into_raw(), lib_callback!(ImageLoaderCallback(self_rc.clone()))));
    }
}

struct ImageLoaderCallback (Rc<RefCell<ImageLoader>>);

lib_define_callback! (ImageLoaderCallback {
    fn callback(&mut self, _ret_code: i32) {
        let mut loader = self.0.borrow_mut();
        assert_eq!(loader.status, ImageLoaderStatus::Loading);
        loader.status = ImageLoaderStatus::Loaded;
        loader.width = lib!(image_get_natural_width(loader.img_id));
        loader.height = lib!(image_get_natural_height(loader.img_id));
        if loader.width > IMAGE_SIZE_WARN {
            warn!("Image width ({}) exceeds max size ({}). May not display properly.", loader.width, IMAGE_SIZE_WARN);
        }
        if loader.height > IMAGE_SIZE_WARN {
            warn!("Image height ({}) exceeds max size ({}). May not display properly.", loader.height, IMAGE_SIZE_WARN);
        }
        let rm = loader.canvas_config.get_resource_manager();
        loader.tex_id = rm.borrow_mut().alloc_tex_id();
        lib!(tex_from_image(loader.canvas_config.index, loader.tex_id, loader.img_id));
        loader.canvas_config.mark_dirty(); // TODO mark connected image dirty but not the whole loader
    }
});

impl Drop for ImageLoader {
    fn drop(&mut self) {
        if self.tex_id != -1 {
            lib!(tex_delete(self.canvas_config.index, self.tex_id));
            let rm = self.canvas_config.get_resource_manager();
            rm.borrow_mut().free_tex_id(self.tex_id);
        }
        lib!(image_unload(self.img_id));
        ResourceManager::free_image_id(self.img_id);
    }
}
