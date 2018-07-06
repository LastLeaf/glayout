use std::ffi::CString;
use std::rc::Rc;
use std::cell::RefCell;
use super::super::CanvasConfig;
use super::super::resource::ResourceManager;
use super::{Element, ElementStyle, BoundingRect};
use super::super::super::tree::{TreeNodeWeak, TreeNodeRc};

const IMAGE_SIZE_WARN: i32 = 4096;

// basic image element

pub struct Image {
    tree_node: Option<TreeNodeWeak<Element>>,
    canvas_config: Rc<CanvasConfig>,
    tex_id: i32,
    loader: Option<Rc<RefCell<ImageLoader>>>,
    natural_width: i32,
    natural_height: i32,
}

impl Image {
    pub fn new(cfg: &Rc<CanvasConfig>) -> Self {
        Image {
            tree_node: None,
            canvas_config: cfg.clone(),
            tex_id: -1,
            loader: None,
            natural_width: 0,
            natural_height: 0,
        }
    }
    fn need_update_from_loader(&mut self) {
        // NOTE this method should be called if manually updated loader
        self.tex_id = -1;
        self.natural_width = 0;
        self.natural_height = 0;
        let mut t = self.tree_node.as_mut().unwrap().upgrade().unwrap();
        t.elem_mut().mark_dirty();
    }
    fn update_from_loader(&mut self) {
        let loader = self.loader.as_ref().unwrap().borrow();
        self.tex_id = loader.tex_id;
        let size = loader.get_size();
        self.natural_width = size.0;
        self.natural_height = size.1;
        let mut t = self.tree_node.as_mut().unwrap().upgrade().unwrap();
        t.elem_mut().mark_dirty();
    }
    pub fn set_loader(&mut self, loader: Rc<RefCell<ImageLoader>>) {
        self.need_update_from_loader();
        match self.loader {
            Some(ref mut loader) => {
                loader.borrow_mut().unbind_tree_node(self.tree_node.clone().unwrap());
            },
            None => { }
        }
        loader.borrow_mut().bind_tree_node(self.tree_node.clone().unwrap());
        self.loader = Some(loader);
    }
    pub fn load<T: Into<Vec<u8>>>(&mut self, url: T) {
        let cc = self.canvas_config.clone();
        self.set_loader(Rc::new(RefCell::new(ImageLoader::new_with_canvas_config(cc))));
        ImageLoader::load(self.loader.as_mut().unwrap().clone(), url);
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        match self.loader {
            Some(ref mut loader) => {
                loader.borrow_mut().unbind_tree_node(self.tree_node.clone().unwrap());
            },
            None => { }
        }
    }
}

impl super::ElementContent for Image {
    fn name(&self) -> &'static str {
        "Image"
    }
    fn associate_tree_node(&mut self, tree_node: TreeNodeRc<Element>) {
        self.tree_node = Some(tree_node.downgrade());
    }
    fn draw(&mut self, style: &ElementStyle, bounding_rect: &BoundingRect) {
        if self.tex_id == -1 {
            return;
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
    LoadFailed,
}

pub struct ImageLoader {
    binded_tree_nodes: Vec<TreeNodeWeak<Element>>,
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
            binded_tree_nodes: vec!(),
            canvas_config: cfg,
            status: ImageLoaderStatus::NotLoaded,
            img_id: ResourceManager::alloc_image_id(),
            tex_id: -1,
            width: 0,
            height: 0,
        }
    }

    pub fn bind_tree_node(&mut self, tree_node: TreeNodeWeak<Element>) {
        self.binded_tree_nodes.push(tree_node)
    }
    pub fn unbind_tree_node(&mut self, tree_node: TreeNodeWeak<Element>) {
        let pos = self.binded_tree_nodes.iter().position(|x| {
            TreeNodeWeak::ptr_eq(x, &tree_node)
        });
        match pos {
            None => { },
            Some(pos) => {
                self.binded_tree_nodes.remove(pos);
            }
        };
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

lib_define_callback! (ImageLoaderCallback (Rc<RefCell<ImageLoader>>) {
    fn callback(&mut self, ret_code: i32) {
        let mut loader = self.0.borrow_mut();
        assert_eq!(loader.status, ImageLoaderStatus::Loading);
        if ret_code == 0 {
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
        } else {
            loader.status = ImageLoaderStatus::LoadFailed;
        }
        lib!(image_unload(loader.img_id));
        ResourceManager::free_image_id(loader.img_id);
        loader.binded_tree_nodes.iter_mut().for_each(|x| {
            let mut t = x.upgrade().unwrap();
            t.elem_mut().content_as_mut::<Image>().update_from_loader();
        });
    }
});

impl Drop for ImageLoader {
    fn drop(&mut self) {
        if self.tex_id != -1 {
            lib!(tex_delete(self.canvas_config.index, self.tex_id));
            let rm = self.canvas_config.get_resource_manager();
            rm.borrow_mut().free_tex_id(self.tex_id);
        }
    }
}
