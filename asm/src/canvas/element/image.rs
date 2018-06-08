use std::ffi::CString;
use super::super::CanvasConfig;
use super::ElementStyle;

// basic image element

#[derive(Debug)]
pub struct Image {
    canvas_index: i32,
    image_id: i32,
    loader: Option<ImageLoader>
}

impl Image {
    pub fn new(cfg: &mut CanvasConfig) -> Self {
        let loader = ImageLoader::new(cfg);
        let image_id = loader.get_id();
        Image {
            canvas_index: cfg.index,
            image_id,
            loader: Some(loader)
        }
    }
    pub fn load<T: Into<Vec<u8>>>(&mut self, url: T) {
        self.loader.take().unwrap().load(url);
    }
}

impl super::ElementContent for Image {
    fn name(&self) -> &'static str {
        "Image"
    }
    fn draw(&self, style: &ElementStyle) {
        // do nothing
        debug!("Attempted to draw an Image");
        if self.loader.is_some() {
            return
        }
        lib!(tex_draw(self.canvas_index, 0, self.image_id, style.left, style.top, style.width, style.height, style.left, style.top, style.width, style.height));
        lib!(tex_draw_end(self.canvas_index));
    }
}

// image loader

#[derive(Debug)]
pub struct ImageLoader {
    canvas_index: i32,
    id: i32
}

impl ImageLoader {
    pub fn new(cfg: &mut CanvasConfig) -> Self {
        ImageLoader {
            canvas_index: cfg.index,
            id: cfg.alloc_image_id()
        }
    }
    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn load<T: Into<Vec<u8>>>(self, url: T) {
        lib!(image_load_url(self.canvas_index, self.id, CString::new(url).unwrap().into_raw(), lib_callback!(self)));
    }
}

lib_define_callback! (ImageLoader {
    fn callback(&mut self, _ret_code: i32) {
        lib!(tex_set_image(self.canvas_index, self.id, self.id, 0., 0., 1024., 1024.)); // TODO
    }
});

impl Drop for ImageLoader {
    fn drop(&mut self) {
        lib!(image_unload(self.canvas_index, self.id));
    }
}
