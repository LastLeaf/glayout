use std::cell::Cell;
use std::rc::Rc;
use std::{f32, f64};
use super::{Element, Transform};
use rc_forest::{ForestNode};

mod types;
pub use self::types::{DisplayType, PositionType, TextAlignType, BoxSizingType};
mod class;
pub use self::class::{StyleName, ElementClass};
mod style_sheet;
pub use self::style_sheet::{StyleSheetGroup, StyleSheet};
mod style_value;

pub const DEFAULT_F64: f64 = f64::INFINITY;
pub const DEFAULT_F32: f32 = f32::INFINITY;

pub struct ElementStyle {
    element: *mut Element,
    inline_class: Cell<ElementClass>,
    classes: Vec<Rc<ElementClass>>,
    tag_name: String,
    id: String,
    class: String,
    display: DisplayType,
    position: PositionType,
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
    width: style_value::F64,
    height: f64,
    font_family: String,
    inherit_font_family: bool,
    font_size: f32,
    inherit_font_size: bool,
    line_height: f32,
    inherit_line_height: bool,
    text_align: TextAlignType,
    inherit_text_align: bool,
    color: (f32, f32, f32, f32),
    inherit_color: bool,
    background_color: (f32, f32, f32, f32),
    opacity: f32,
    transform: Transform,
    margin_left: f64,
    margin_right: f64,
    margin_top: f64,
    margin_bottom: f64,
    padding_left: f64,
    padding_right: f64,
    padding_top: f64,
    padding_bottom: f64,
    box_sizing: BoxSizingType,
    border_left_width: f64,
    border_right_width: f64,
    border_top_width: f64,
    border_bottom_width: f64,
    border_left_color: (f32, f32, f32, f32),
    border_right_color: (f32, f32, f32, f32),
    border_top_color: (f32, f32, f32, f32),
    border_bottom_color: (f32, f32, f32, f32),
}

impl Default for ElementStyle {
    fn default() -> Self {
        ElementStyle {
            element: 0 as *mut Element,
            inline_class: Cell::new(ElementClass::new()),
            classes: vec![],
            tag_name: String::new(),
            id: String::new(),
            class: String::new(),
            display: DisplayType::Inline,
            position: PositionType::Static,
            left: DEFAULT_F64,
            top: DEFAULT_F64,
            right: DEFAULT_F64,
            bottom: DEFAULT_F64,
            width: DEFAULT_F64,
            height: DEFAULT_F64,
            font_family: String::from("sans-serif"),
            inherit_font_family: true,
            font_size: 16.,
            inherit_font_size: true,
            line_height: DEFAULT_F32,
            inherit_line_height: true,
            text_align: TextAlignType::Left,
            inherit_text_align: true,
            color: (0., 0., 0., 1.),
            inherit_color: true,
            background_color: (-1., -1., -1., -1.),
            opacity: 1.,
            transform: Transform::new(),
            margin_left: 0.,
            margin_right: 0.,
            margin_top: 0.,
            margin_bottom: 0.,
            padding_left: 0.,
            padding_right: 0.,
            padding_top: 0.,
            padding_bottom: 0.,
            box_sizing: BoxSizingType::ContentBox,
            border_left_width: 0.,
            border_right_width: 0.,
            border_top_width: 0.,
            border_bottom_width: 0.,
            border_left_color: (0., 0., 0., 0.),
            border_right_color: (0., 0., 0., 0.),
            border_top_color: (0., 0., 0., 0.),
            border_bottom_color: (0., 0., 0., 0.),
        }
    }
}

impl ElementStyle {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }
}

macro_rules! getter_setter {
    ($name:ident, $getter:ident, $setter:ident, $type:ty) => {
        #[inline]
        pub fn $getter(&self) -> $type {
            self.$name.clone()
        }
        #[inline]
        pub fn $name(&mut self, val: $type) {
            self.inline_class.get_mut().replace_rule(StyleName::$name, Box::new(val.clone()));
            self.$setter(val);
        }
        #[inline]
        pub(self) fn $setter(&mut self, val: $type) {
            self.$name = val;
        }
    }
}
macro_rules! getter_setter_layout_dirty {
    ($name:ident, $getter:ident, $setter:ident, $type:ty) => {
        #[inline]
        pub fn $getter(&self) -> $type {
            self.$name.clone()
        }
        #[inline]
        pub fn $name(&mut self, val: $type) {
            if self.$name == val { return }
            self.inline_class.get_mut().replace_rule(StyleName::$name, Box::new(val.clone()));
            self.$setter(val);
        }
        #[inline]
        pub(self) fn $setter(&mut self, val: $type) {
            self.element_mut().mark_layout_dirty();
            self.$name = val;
        }
    }
}
macro_rules! getter_setter_inherit_layout_dirty {
    ($name:ident, $getter:ident, $setter:ident, $inherit_name:ident, $inherit_getter:ident, $dfs_setter:ident, $type:ty) => {
        #[inline]
        pub fn $getter(&self) -> $type {
            self.$name.clone()
        }
        #[inline]
        pub fn $name(&mut self, val: $type) {
            self.inline_class.get_mut().replace_rule(StyleName::$name, Box::new(val.clone()));
            self.$setter(val);
        }
        #[inline]
        pub(self) fn $setter(&mut self, val: $type) {
            self.$inherit_name = false;
            self.$dfs_setter(val);
        }
        fn $dfs_setter(&mut self, val: $type) {
            if self.$name == val { return }
            let s = unsafe { self.clone_mut_unsafe() };
            let tree_node = self.node_mut();
            tree_node.mark_layout_dirty();
            s.$name = val.clone();
            tree_node.for_each_child_mut(|child| {
                let style = child.style_mut();
                if style.$inherit_name {
                    style.$dfs_setter(val.clone());
                }
            })
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
                    x.style().$name.clone()
                },
            });
        }
    }
}

impl ElementStyle {
    getter_setter_layout_dirty!(display, get_display, set_display, DisplayType);
    getter_setter_layout_dirty!(position, get_position, set_position, PositionType);
    getter_setter_layout_dirty!(left, get_left, set_left, f64);
    getter_setter_layout_dirty!(top, get_top, set_top, f64);
    getter_setter_layout_dirty!(right, get_right, set_right, f64);
    getter_setter_layout_dirty!(bottom, get_bottom, set_bottom, f64);
    getter_setter_layout_dirty!(width, get_width, set_width, f64);
    getter_setter_layout_dirty!(height, get_height, set_height, f64);
    getter_setter_inherit_layout_dirty!(font_family, get_font_family, set_font_family, inherit_font_family, get_inherit_font_family, inherit_font_family, String);
    getter_setter_inherit_layout_dirty!(font_size, get_font_size, set_font_size, inherit_font_size, get_inherit_font_size, inherit_font_size, f32);
    getter_setter_inherit_layout_dirty!(line_height, get_line_height, set_line_height, inherit_line_height, get_inherit_line_height, inherit_line_height, f32);
    getter_setter_inherit_layout_dirty!(text_align, get_text_align, set_text_align, inherit_text_align, get_inherit_text_align, inherit_text_align, TextAlignType);
    // FIXME changing color does not need mark dirty
    getter_setter_inherit_layout_dirty!(color, get_color, set_color, inherit_color, get_inherit_color, inherit_color, (f32, f32, f32, f32));
    getter_setter!(background_color, get_background_color, set_background_color, (f32, f32, f32, f32));
    getter_setter!(opacity, get_opacity, set_opacity, f32);
    getter_setter!(transform, get_transform, set_transform, Transform);
    getter_setter_layout_dirty!(margin_left, get_margin_left, set_margin_left, f64);
    getter_setter_layout_dirty!(margin_right, get_margin_right, set_margin_right, f64);
    getter_setter_layout_dirty!(margin_top, get_margin_top, set_margin_top, f64);
    getter_setter_layout_dirty!(margin_bottom, get_margin_bottom, set_margin_bottom, f64);
    getter_setter_layout_dirty!(padding_left, get_padding_left, set_padding_left, f64);
    getter_setter_layout_dirty!(padding_right, get_padding_right, set_padding_right, f64);
    getter_setter_layout_dirty!(padding_top, get_padding_top, set_padding_top, f64);
    getter_setter_layout_dirty!(padding_bottom, get_padding_bottom, set_padding_bottom, f64);
    getter_setter_layout_dirty!(box_sizing, get_box_sizing, set_box_sizing, BoxSizingType);
    getter_setter_layout_dirty!(border_left_width, get_border_left_width, set_border_left_width, f64);
    getter_setter_layout_dirty!(border_right_width, get_border_right_width, set_border_right_width, f64);
    getter_setter_layout_dirty!(border_top_width, get_border_top_width, set_border_top_width, f64);
    getter_setter_layout_dirty!(border_bottom_width, get_border_bottom_width, set_border_bottom_width, f64);
    getter_setter!(border_left_color, get_border_left_color, set_border_left_color, (f32, f32, f32, f32));
    getter_setter!(border_right_color, get_border_right_color, set_border_right_color, (f32, f32, f32, f32));
    getter_setter!(border_top_color, get_border_top_color, set_border_top_color, (f32, f32, f32, f32));
    getter_setter!(border_bottom_color, get_border_bottom_color, set_border_bottom_color, (f32, f32, f32, f32));

    fn update_inherit(&mut self, parent_node: Option<&mut ForestNode<Element>>) {
        update_inherit!(self, parent_node, font_family, inherit_font_family, inherit_font_family, String::from("sans-serif"));
        update_inherit!(self, parent_node, font_size, inherit_font_size, inherit_font_size, 16.);
        update_inherit!(self, parent_node, color, inherit_color, inherit_color, (0., 0., 0., 1.));
    }

    pub fn get_tag_name(&self) -> String {
        self.tag_name.clone()
    }
    pub fn tag_name(&mut self, s: String) {
        self.tag_name = s;
        self.reload_classes();
    }
    pub fn get_id(&self) -> String {
        self.id.clone()
    }
    pub fn id(&mut self, s: String) {
        self.id = s;
        self.reload_classes();
    }
    pub fn get_class(&self) -> String {
        self.class.clone()
    }
    pub fn class(&mut self, s: String) {
        self.class = s;
        self.reload_classes();
    }
    fn reload_classes(&mut self) {
        // FIXME only do queries when needed
        let s = unsafe { self.clone_ref_unsafe() };
        self.classes = self.node_mut().canvas_config.query_classes(&s.tag_name, &s.id, &s.class);

        // FIXME if there is a class removed, here is no reset
        {
            let cs = self.classes.clone();
            for c in cs {
                c.apply_to_style(self);
            }
        }
        let c = self.inline_class.take();
        c.apply_to_style(self);
        self.inline_class.set(c);
    }
    pub(super) fn inline_text(&mut self, text: &str) {
        let mut c = ElementClass::new();
        StyleSheet::parse_inline_style(&mut c, text);
        self.inline_class.set(c);
        self.reload_classes();
    }
}

impl ElementStyle {
    #[inline]
    pub fn associate_element(&mut self, element: *mut Element) {
        self.element = element;
    }
    #[inline]
    fn element_mut<'a>(&'a mut self) -> &'a mut Element {
        unsafe { &mut *self.element }
    }
    #[inline]
    fn node_mut<'a>(&'a mut self) -> &'a mut ForestNode<Element> {
        self.element_mut().node_mut()
    }
    #[inline]
    pub(super) unsafe fn clone_ref_unsafe<'a, 'b>(&'a self) -> &'b Self {
        &*(self as *const Self)
    }
    #[inline]
    pub(super) unsafe fn clone_mut_unsafe<'a, 'b>(&'a mut self) -> &'b mut Self {
        &mut *(self as *mut Self)
    }
    #[inline]
    pub fn parent_node_changed(&mut self) {
        let s = unsafe { self.clone_mut_unsafe() };
        s.update_inherit(self.node_mut().parent_mut());
    }
    #[inline]
    pub fn transform_ref(&self) -> &Transform {
        &self.transform
    }
    #[inline]
    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}
