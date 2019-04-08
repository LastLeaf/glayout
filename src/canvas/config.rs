use std::rc::Rc;
use std::cell::{Cell, RefCell};
use super::character::CharacterManager;
use super::resource::ResourceManager;
use super::element::style::{StyleSheetGroup, StyleSheet, ElementClass};
use super::element::{Element, Size};
use rc_forest::{ForestNode, ForestNodeWeak};

pub struct CanvasConfig {
    pub index: i32,
    pub tex_size: i32,
    pub tex_count: i32,
    pub tex_max_draws: i32,
    pub device_pixel_ratio: f64,
    pub canvas_size: Cell<Size>,
    root_node: RefCell<Option<ForestNodeWeak<Element>>>,
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

    fn mark_class_dirty_from_root(&self, some_node: Option<&mut ForestNode<Element>>) {
        let mut root_node = self.root_node.borrow_mut();
        match root_node.as_mut() {
            None => { },
            Some(w) => {
                match w.upgrade() {
                    None => { },
                    Some(rc) => {
                        match some_node {
                            Some(some_node) => {
                                rc.deref_mut_with(some_node).mark_class_dirty_dfs();
                            },
                            None => {
                                rc.borrow_mut().mark_class_dirty_dfs();
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn append_style_sheet(&self, some_node: &mut ForestNode<Element>, css_text: &str) -> usize {
        let ss = StyleSheet::new_from_css(css_text);
        let ret = self.style_sheet_group.borrow_mut().len();
        self.style_sheet_group.borrow_mut().append(ss);
        self.mark_class_dirty_from_root(Some(some_node));
        ret
    }
    pub fn append_style_sheet_alone(&self, css_text: &str) -> usize {
        let ss = StyleSheet::new_from_css(css_text);
        let ret = self.style_sheet_group.borrow_mut().len();
        self.style_sheet_group.borrow_mut().append(ss);
        self.mark_class_dirty_from_root(None);
        ret
    }
    pub fn replace_style_sheet(&self, some_node: &mut ForestNode<Element>, index: usize, css_text: &str) {
        let ss = StyleSheet::new_from_css(css_text);
        self.style_sheet_group.borrow_mut().replace(index, ss);
        self.mark_class_dirty_from_root(Some(some_node));
    }
    pub fn replace_style_sheet_alone(&self, index: usize, css_text: &str) {
        let ss = StyleSheet::new_from_css(css_text);
        self.style_sheet_group.borrow_mut().replace(index, ss);
        self.mark_class_dirty_from_root(None);
    }
    pub fn clear_style_sheets(&self) {
        self.style_sheet_group.borrow_mut().clear();
    }
    #[inline]
    pub fn query_classes(&self, tag_name: &str, id: &str, class_names: &str) -> Vec<Rc<ElementClass>> {
        self.style_sheet_group.borrow().query_declarations(tag_name, id, class_names.split_whitespace().collect())
    }
}
