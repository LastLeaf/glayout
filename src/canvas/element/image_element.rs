use std::ffi::CString;
use std::rc::Rc;
use std::cell::RefCell;
use super::super::CanvasConfig;
use super::super::resource::ResourceManager;
use super::{Element, ElementStyle, InlineAllocator, Transform, Position, Bounds, Size, Point};
use super::style::{DEFAULT_F64};
use rc_forest::{ForestNode, ForestNodeWeak};

const IMAGE_SIZE_WARN: i32 = 4096;

// basic image element

pub struct Image {
    element: *mut Element,
    canvas_config: Rc<CanvasConfig>,
    tex_id: i32,
    loader: Option<Rc<RefCell<ImageLoader>>>,
    inline_pos: Position,
    natural_size: (i32, i32),
}

impl Image {
    pub fn new(cfg: &Rc<CanvasConfig>) -> Self {
        Image {
            element: 0 as *mut Element,
            canvas_config: cfg.clone(),
            tex_id: -1,
            loader: None,
            inline_pos: Position::new(0., 0., 0., 0.),
            natural_size: (0, 0),
        }
    }
    #[inline]
    fn element<'a>(&'a self) -> &'a Element {
        unsafe { &*self.element }
    }
    #[inline]
    fn node<'a>(&'a self) -> &'a ForestNode<Element> {
        self.element().node()
    }
    #[inline]
    fn element_mut<'a>(&'a mut self) -> &'a mut Element {
        unsafe { &mut *self.element }
    }
    #[inline]
    fn node_mut<'a>(&'a mut self) -> &'a mut ForestNode<Element> {
        self.element_mut().node_mut()
    }

    fn need_update_from_loader(&mut self) {
        // NOTE this method should be called if manually updated loader
        self.tex_id = -1;
        self.natural_size = (0, 0);
        self.element_mut().mark_layout_dirty();
    }
    fn update_from_loader(&mut self) {
        {
            let loader = self.loader.as_ref().unwrap().borrow();
            self.tex_id = loader.tex_id;
            let size = loader.size();
            self.natural_size = size;
        }
        self.element_mut().mark_layout_dirty();
    }
    pub fn set_loader(&mut self, loader: Rc<RefCell<ImageLoader>>) {
        self.need_update_from_loader();
        let rc = self.node().rc();
        match self.loader {
            Some(ref mut loader) => {
                loader.borrow_mut().unbind_tree_node(rc.downgrade());
            },
            None => { }
        }
        let loader_loaded = {
            let loader = loader.borrow();
            match loader.status {
                ImageLoaderStatus::Loaded | ImageLoaderStatus::LoadFailed => {
                    true
                },
                ImageLoaderStatus::NotLoaded | ImageLoaderStatus::Loading => {
                    false
                }
            }
        };
        {
            loader.borrow_mut().bind_tree_node(self.node().rc().downgrade());
            self.loader = Some(loader);
        }
        if loader_loaded {
            self.update_from_loader();
        }
    }
    pub fn load<T: Into<Vec<u8>>>(&mut self, url: T) {
        let cc = self.canvas_config.clone();
        self.set_loader(Rc::new(RefCell::new(ImageLoader::new_with_canvas_config(cc))));
        ImageLoader::load(self.loader.as_mut().unwrap().clone(), url);
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        let rc = self.node().rc();
        match self.loader {
            Some(ref mut loader) => {
                loader.borrow_mut().unbind_tree_node(rc.downgrade());
            },
            None => { }
        }
    }
}

impl super::ElementContent for Image {
    #[inline]
    fn name(&self) -> &'static str {
        "Image"
    }
    #[inline]
    fn is_terminated(&self) -> bool {
        true
    }
    fn clone(&self) -> Box<super::ElementContent> {
        let cfg = &self.canvas_config;
        let mut ret = Box::new(Image {
            element: 0 as *mut Element,
            canvas_config: cfg.clone(),
            tex_id: self.tex_id,
            loader: None,
            inline_pos: Position::new(0., 0., 0., 0.),
            natural_size: self.natural_size,
        });
        match self.loader.clone() {
            None => {},
            Some(loader) => {
                ret.set_loader(loader.clone());
            }
        }
        ret
    }
    #[inline]
    fn associate_element(&mut self, element: *mut Element) {
        self.element = element;
    }
    fn suggest_size(&mut self, suggested_size: Size, inline_allocator: &mut InlineAllocator, style: &ElementStyle) -> Size {
        let base_requested_top = inline_allocator.get_current_height();
        let spec_width = style.get_width() != DEFAULT_F64;
        let spec_height = style.get_height() != DEFAULT_F64;
        let width;
        let height;
        if spec_width {
            if spec_height {
                width = style.get_width();
                height = style.get_height();
            } else {
                width = style.get_width();
                if self.natural_size.0 == 0 { height = 0.; }
                else { height = width / self.natural_size.0 as f64 * self.natural_size.1 as f64; }
            }
        } else {
            if spec_height {
                height = style.get_height();
                if self.natural_size.1 == 0 { width = 0.; }
                else { width = height / self.natural_size.1 as f64 * self.natural_size.0 as f64; }
            } else {
                width = self.natural_size.0 as f64;
                height = self.natural_size.1 as f64;
            }
        }
        let baseline_top = height / 2.; // FIXME vertical-align middle
        inline_allocator.start_node(self.node_mut(), height, baseline_top);
        let (left, line_baseline_top) = inline_allocator.add_width(self.node_mut(), width, true).into();
        self.inline_pos = Position::new(left, line_baseline_top - baseline_top - base_requested_top, width, height);
        Size::new(suggested_size.width(), height - base_requested_top)
    }
    #[inline]
    fn adjust_baseline_offset(&mut self, add_offset: f64) {
        self.inline_pos.move_size(Size::new(0., add_offset));
    }
    #[inline]
    fn adjust_text_align_offset(&mut self, add_offset: f64) {
        self.inline_pos.move_size(Size::new(add_offset, 0.));
    }
    fn draw(&mut self, transform: &Transform) {
        if self.tex_id == -1 {
            return;
        }
        let rm = self.canvas_config.resource_manager();
        rm.borrow_mut().request_draw(
            self.tex_id, false,
            0., 0., 1., 1.,
            transform.apply_to_position(&self.inline_pos).into()
        );
    }
    #[inline]
    fn drawing_bounds(&self) -> Bounds {
        self.inline_pos.into()
        // TODO fix inline bounding bug
    }
    fn is_under_point(&self, point: Point, transform: Transform) -> bool {
        if self.tex_id == -1 {
            return false;
        }
        let pos = transform.apply_to_position(&self.inline_pos);
        // debug!("testing {:?} in image pos {:?}", (x, y), pos);
        point.in_position(&pos)
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
    id: String,
    binded_tree_nodes: Vec<ForestNodeWeak<Element>>,
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
            id: String::new(),
            binded_tree_nodes: vec!(),
            canvas_config: cfg,
            status: ImageLoaderStatus::NotLoaded,
            img_id: ResourceManager::alloc_image_id(),
            tex_id: -1,
            width: 0,
            height: 0,
        }
    }

    #[inline]
    pub fn bind_tree_node(&mut self, tree_node: ForestNodeWeak<Element>) {
        self.binded_tree_nodes.push(tree_node)
    }
    pub fn unbind_tree_node(&mut self, tree_node: ForestNodeWeak<Element>) {
        let pos = self.binded_tree_nodes.iter().position(|x| {
            ForestNodeWeak::ptr_eq(x, &tree_node)
        });
        match pos {
            None => { },
            Some(pos) => {
                self.binded_tree_nodes.remove(pos);
            }
        };
    }

    #[inline]
    pub fn id(&self) -> String {
        self.id.clone()
    }
    #[inline]
    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }
    #[inline]
    pub fn img_id(&self) -> i32 {
        self.img_id
    }
    #[inline]
    pub fn status(&self) -> ImageLoaderStatus {
        self.status
    }
    #[inline]
    pub fn is_loading(&self) -> bool {
        self.status == ImageLoaderStatus::Loading
    }
    #[inline]
    pub fn size(&self) -> (i32, i32) {
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
    fn callback(&mut self, ret_code: i32, _: i32, _: i32, _: i32) -> bool {
        let mut nodes = {
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
                let rm = loader.canvas_config.resource_manager();
                loader.tex_id = rm.borrow_mut().alloc_tex_id();
                log!("Image loaded: {}", loader.img_id);
                lib!(tex_from_image(loader.canvas_config.index, loader.tex_id, loader.img_id));
            } else {
                loader.status = ImageLoaderStatus::LoadFailed;
            }
            lib!(image_unload(loader.img_id));
            ResourceManager::free_image_id(loader.img_id);
            loader.binded_tree_nodes.clone()
        };
        nodes.iter_mut().for_each(|x| {
            match x.upgrade() {
                None => { },
                Some(x) => {
                    x.borrow_mut().downcast_mut::<Image>().unwrap().update_from_loader();
                }
            }
        });
        false
    }
});

impl Drop for ImageLoader {
    fn drop(&mut self) {
        if self.tex_id != -1 {
            lib!(tex_delete(self.canvas_config.index, self.tex_id));
            let rm = self.canvas_config.resource_manager();
            rm.borrow_mut().free_tex_id(self.tex_id);
        }
    }
}
