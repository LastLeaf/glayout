use std::rc::Rc;
use std::cell::{Cell, RefCell};
use super::character::CharacterManager;
use super::resource::ResourceManager;
use super::element::style::{StyleSheetGroup, StyleSheet, ElementClass};
use super::element::{Element, Size};
use rc_forest::ForestNodeWeak;

pub struct CanvasConfig {
    pub index: i32,
    pub tex_size: i32,
    pub tex_count: i32,
    pub tex_max_draws: i32,
    pub device_pixel_ratio: f64,
    pub canvas_size: Cell<Size>,
    root_node: RefCell<Option<ForestNodeWeak<Element>>>,
    style_sheet_dirty: Cell<bool>,
    clear_color: Cell<(f32, f32, f32, f32)>,
    resource_manager: Rc<RefCell<ResourceManager>>,
    character_manager: Rc<RefCell<CharacterManager>>,
    style_sheet_group: RefCell<StyleSheetGroup>,
}

impl CanvasConfig {
    pub fn new(index: i32, tex_size: i32, tex_count: i32, tex_max_draws: i32, device_pixel_ratio: f64) -> Self {
        let resource_manager = Rc::new(RefCell::new(ResourceManager::new(index, tex_max_draws)));
        CanvasConfig {
            index,
            tex_size,
            tex_count,
            tex_max_draws,
            device_pixel_ratio,
            canvas_size: Cell::new(Size::new(1280., 720.)),
            root_node: RefCell::new(None),
            style_sheet_dirty: Cell::new(false),
            clear_color: Cell::new((1., 1., 1., 0.)),
            resource_manager: resource_manager.clone(),
            character_manager: Rc::new(RefCell::new(CharacterManager::new(index, resource_manager))),
            style_sheet_group: RefCell::new(StyleSheetGroup::new()),
        }
    }
    pub(super) fn root_node<'a>(&'a self) -> Option<ForestNodeWeak<Element>> {
        self.root_node.borrow().clone()
    }
    pub(super) fn set_root_node(&self, weak: ForestNodeWeak<Element>) {
        let mut x = self.root_node.borrow_mut();
        *x = Some(weak);
    }

    #[inline]
    pub fn set_clear_color(&self, color: (f32, f32, f32, f32)) {
        self.clear_color.set(color);
    }
    #[inline]
    pub fn clear_color(&self) -> (f32, f32, f32, f32) {
        self.clear_color.get()
    }

    #[inline]
    pub fn character_manager(&self) -> Rc<RefCell<CharacterManager>> {
        self.character_manager.clone()
    }
    #[inline]
    pub fn resource_manager(&self) -> Rc<RefCell<ResourceManager>> {
        self.resource_manager.clone()
    }

    #[inline]
    pub(crate) fn clear_style_sheet_dirty(&self) -> bool {
        self.style_sheet_dirty.replace(false)
    }
    #[inline]
    pub fn append_style_sheet(&self, css_text: &str) {
        let ss = StyleSheet::new_from_css(css_text);
        self.style_sheet_group.borrow_mut().append(ss);
    }
    #[inline]
    pub fn clear_style_sheets(&self) {
        self.style_sheet_group.borrow_mut().clear();
    }
    #[inline]
    pub fn query_classes(&self, tag_name: &str, id: &str, class_names: &str) -> Vec<Rc<ElementClass>> {
        self.style_sheet_group.borrow().query_declarations(tag_name, id, class_names.split_whitespace().collect())
    }
}
