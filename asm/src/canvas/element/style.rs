use std::f64;
use super::{Element};
use super::super::super::tree::{TreeNodeWeak};

const DEFAULT_F64: f64 = f64::INFINITY;

pub struct ElementStyle {
    tree_node: Option<TreeNodeWeak<Element>>,
    id: String,
    pub display: DisplayType,
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub font_family: String,
    pub font_size: f64,
    pub color: (f32, f32, f32, f32),
}

impl ElementStyle {
    pub fn new() -> Self {
        ElementStyle {
            tree_node: None,
            id: String::new(),
            display: DisplayType::Inline,
            left: DEFAULT_F64,
            top: DEFAULT_F64,
            width: DEFAULT_F64,
            height: DEFAULT_F64,
            font_family: String::from("sans-serif"),
            font_size: 16.,
            color: (0., 0., 0., 0.),
        }
    }
}

macro_rules! getter_setter {
    ($name:ident, $getter:ident, $setter:ident, $type:path) => {
        pub fn $getter(&self) -> $type {
            self.$name.clone()
        }
        pub fn $setter(&mut self, val: $type) {
            self.$name = val;
        }
    }
}
macro_rules! getter_setter_dirty {
    ($name:ident, $getter:ident, $setter:ident, $type:path) => {
        pub fn $getter(&self) -> $type {
            self.$name.clone()
        }
        pub fn $setter(&mut self, val: $type) {
            self.tree_node.as_ref().unwrap().upgrade().unwrap().elem().mark_dirty();
            self.$name = val;
        }
    }
}

impl ElementStyle {
    getter_setter!(id, get_id, id, String);
    getter_setter_dirty!(display, get_display, display, DisplayType);
    getter_setter_dirty!(left, get_left, left, f64);
    getter_setter_dirty!(top, get_top, top, f64);
    getter_setter_dirty!(width, get_width, width, f64);
    getter_setter_dirty!(height, get_height, height, f64);
    getter_setter_dirty!(font_family, get_font_family, font_family, String);
    getter_setter_dirty!(font_size, get_font_size, font_size, f64);
}

impl ElementStyle {
    pub fn associate_tree_node(&mut self, tree_node: TreeNodeWeak<Element>) {
        self.tree_node = Some(tree_node);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum DisplayType {
    None,
    Block,
    Inline,
    InlineBlock,
    Flex,
}
