use std::f64;
use super::{Element, Transform};
use super::super::super::tree::{TreeNodeRc, TreeNodeWeak};

const DEFAULT_F64: f64 = f64::INFINITY;

pub struct ElementStyle {
    tree_node: Option<TreeNodeWeak<Element>>,
    id: String,
    display: DisplayType,
    position: PositionType,
    left: f64,
    top: f64,
    width: f64,
    height: f64,
    font_family: String,
    inherit_font_family: bool,
    font_size: f64,
    inherit_font_size: bool,
    color: (f32, f32, f32, f32),
    inherit_color: bool,
    transform: Transform,
}

impl ElementStyle {
    pub fn new() -> Self {
        ElementStyle {
            tree_node: None,
            id: String::new(),
            display: DisplayType::Inline,
            position: PositionType::Static,
            left: DEFAULT_F64,
            top: DEFAULT_F64,
            width: DEFAULT_F64,
            height: DEFAULT_F64,
            font_family: String::from("sans-serif"),
            inherit_font_family: true,
            font_size: 16.,
            inherit_font_size: true,
            color: (0., 0., 0., 1.),
            inherit_color: true,
            transform: Transform::new(),
        }
    }
}

macro_rules! getter_setter {
    ($name:ident, $getter:ident, $setter:ident, $type:ty) => {
        #[inline]
        pub fn $getter(&self) -> $type {
            self.$name.clone()
        }
        #[inline]
        pub fn $setter(&mut self, val: $type) {
            self.$name = val;
        }
    }
}
macro_rules! getter_setter_dirty {
    ($name:ident, $getter:ident, $setter:ident, $type:ty) => {
        #[inline]
        pub fn $getter(&self) -> $type {
            self.$name.clone()
        }
        #[inline]
        pub fn $setter(&mut self, val: $type) {
            if self.$name == val { return }
            self.tree_node().elem().mark_dirty();
            self.$name = val;
        }
    }
}
macro_rules! getter_setter_inherit_dirty {
    ($name:ident, $getter:ident, $setter:ident, $inherit_name:ident, $inherit_getter:ident, $dfs_setter:ident, $type:ty) => {
        #[inline]
        pub fn $getter(&self) -> $type {
            self.$name.clone()
        }
        #[inline]
        pub fn $setter(&mut self, val: $type) {
            self.$inherit_name = false;
            self.$dfs_setter(val);
        }
        fn $dfs_setter(&mut self, val: $type) {
            if self.$name == val { return }
            let tree_node = self.tree_node();
            tree_node.elem().mark_dirty();
            self.$name = val.clone();
            for child in tree_node.iter_children() {
                let mut style = child.elem().style_mut();
                if style.$inherit_name {
                    style.$dfs_setter(val.clone());
                }
            }
        }
        #[inline]
        pub fn $inherit_getter(&self) -> bool {
            self.$inherit_name
        }
    }
}
macro_rules! update_inherit {
    ($self:ident, $parent_node:ident, $name:ident, $inherit_name:ident, $dfs_setter:ident, $default:expr) => {
        if $self.$inherit_name {
            $self.$dfs_setter(match &$parent_node {
                None => {
                    $default
                },
                Some(ref x) => {
                    x.elem().style().$name.clone()
                },
            });
        }
    }
}

impl ElementStyle {
    getter_setter!(id, get_id, id, String);
    getter_setter_dirty!(display, get_display, display, DisplayType);
    getter_setter_dirty!(position, get_position, position, PositionType);
    getter_setter_dirty!(left, get_left, left, f64);
    getter_setter_dirty!(top, get_top, top, f64);
    getter_setter_dirty!(width, get_width, width, f64);
    getter_setter_dirty!(height, get_height, height, f64);
    getter_setter_inherit_dirty!(font_family, get_font_family, font_family, inherit_font_family, get_inherit_font_family, inherit_font_family, String);
    getter_setter_inherit_dirty!(font_size, get_font_size, font_size, inherit_font_size, get_inherit_font_size, inherit_font_size, f64);
    // FIXME changing color does not need mark dirty
    getter_setter_inherit_dirty!(color, get_color, color, inherit_color, get_inherit_color, inherit_color, (f32, f32, f32, f32));
    getter_setter!(transform, get_transform, transform, Transform);

    fn update_inherit(&mut self, parent_node: Option<TreeNodeRc<Element>>) {
        update_inherit!(self, parent_node, font_family, inherit_font_family, inherit_font_family, String::from("sans-serif"));
        update_inherit!(self, parent_node, font_size, inherit_font_size, inherit_font_size, 16.);
        update_inherit!(self, parent_node, color, inherit_color, inherit_color, (0., 0., 0., 1.));
    }
}

impl ElementStyle {
    #[inline]
    pub fn associate_tree_node(&mut self, tree_node: TreeNodeWeak<Element>) {
        self.tree_node = Some(tree_node);
    }
    #[inline]
    pub fn parent_node_changed(&mut self, parent_node: Option<TreeNodeRc<Element>>) {
        self.update_inherit(parent_node);
    }
    #[inline]
    fn tree_node(&self) -> TreeNodeRc<Element> {
        self.tree_node.as_ref().unwrap().upgrade().unwrap()
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

#[derive(Clone, Copy, PartialEq)]
pub enum PositionType {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}
