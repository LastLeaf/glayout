use std::cell::Cell;
use std::rc::Rc;
use std::{f32, f64};
use super::{Element, Transform};
use rc_forest::{ForestNode};
use glayout_element_style_macro::*;

mod types;
pub use self::types::*;
mod class;
pub use self::class::{StyleName, ElementClass};
mod style_sheet;
pub use self::style_sheet::{StyleSheetGroup, StyleSheet};
mod style_value;
use self::style_value::{StyleValue, StyleValueReferrer};

pub const DEFAULT_F64: f64 = f64::INFINITY;
pub const DEFAULT_F32: f32 = f32::INFINITY;

macro_rules! define_struct {
    ($($items:tt)*) => {
        pub struct ElementStyle {
            element: *mut Element,
            inline_class: Cell<ElementClass>,
            classes: Vec<Rc<ElementClass>>,
            tag_name: String,
            id: String,
            class: String,
            $($items)*
        }
    }
}

macro_rules! define_constructor {
    ($($items:tt)*) => {
        impl ElementStyle {
            pub(crate) fn new() -> Self {
                Self {
                    element: 0 as *mut Element,
                    inline_class: Cell::new(ElementClass::new()),
                    classes: vec![],
                    tag_name: String::new(),
                    id: String::new(),
                    class: String::new(),
                    $($items)*
                }
            }
        }
    }
}

macro_rules! impl_style_item {
    (
        $name:ident,
        $getter:ident,
        $setter:ident,
        $getter_advanced:ident,
        $setter_advanced:ident,
        $getter_inner:ident,
        $setter_inner:ident,
        $update_inherit:ident,
        $value_type:ty,
        $default_value_referrer:expr,
        $default_value:expr,
        $layout_dirty:expr,
        $inherit:expr
    ) => {
        pub(self) fn $getter_inner(&self) -> (StyleValueReferrer, $value_type) {
            if self.$name.is_dirty() {
                if self.$name.inherit() {
                    let value = {
                        let tree_node = self.node();
                        let parent = tree_node.parent();
                        match parent {
                            Some(p) => p.style.$name.get(),
                            None => ($default_value_referrer, $default_value),
                        }
                    };
                    self.$name.set(value.0, value.1);
                }
                self.$name.clear_dirty();
            }
            self.$name.get()
        }
        #[inline]
        pub fn $getter_advanced(&self) -> (StyleValueReferrer, $value_type) {
            self.$getter_inner()
        }
        #[inline]
        pub fn $getter(&self) -> $value_type {
            self.$getter_advanced().1
        }
        fn $update_inherit(tree_node: &mut ForestNode<Element>) {
            if $layout_dirty { tree_node.mark_layout_dirty() };
            let old_dirty = tree_node.style.$name.get_and_mark_dirty();
            if !old_dirty {
                tree_node.for_each_child_mut(|child| {
                    if child.style.$name.inherit() {
                        Self::$update_inherit(child);
                    }
                })
            }
        }
        pub(self) fn $setter_inner(&mut self, r: StyleValueReferrer, val: $value_type, inherit: bool) {
            let val = if r.is_absolute_or_relative() {
                val
            } else {
                $default_value
            };
            let changed = if inherit {
                let changed = !self.$name.inherit();
                self.$name.set_inherit(true);
                changed
            } else {
                let changed = r == self.$name.get_referrer() && val == *self.$name.get_value_ref();
                self.$name.set(r, val);
                changed
            };
            if changed {
                let tree_node = self.node_mut();
                Self::$update_inherit(tree_node);
            }
        }
        #[inline]
        pub fn $setter_advanced(&mut self, r: StyleValueReferrer, val: $value_type, inherit: bool) {
            self.inline_class.get_mut().replace_rule(StyleName::$name, Box::new(val.clone()));
            self.$setter_inner(r, val, inherit);
        }
        #[inline]
        pub fn $setter(&mut self, val: $value_type) {
            self.$setter_advanced(StyleValueReferrer::Absolute, val, false);
        }
        #[inline]
        pub fn $name(&mut self, val: $value_type) {
            self.$setter(val);
        }
    }
}

macro_rules! impl_style_list {
    ($($items:tt)*) => {
        impl ElementStyle {
            $($items)*
        }
    }
}

macro_rules! impl_parent_updated_item {
    ($s:ident, $name:ident, $update_inherit:ident) => {
        if $s.$name.inherit() {
            Self::$update_inherit($s.node_mut());
        }
    }
}

macro_rules! impl_parent_updated {
    ($($items:tt)*) => {
        impl ElementStyle {
            $($items)*
        }
    }
}

element_style! {
    display: DisplayType, Absolute(DisplayType::Inline), (layout_dirty);
    position: PositionType, Absolute(PositionType::Static), (layout_dirty);
    left: f64, Auto(DEFAULT_F64), (layout_dirty);
    right: f64, Auto(DEFAULT_F64), (layout_dirty);
    top: f64, Auto(DEFAULT_F64), (layout_dirty);
    bottom: f64, Auto(DEFAULT_F64), (layout_dirty);
    width: f64, Auto(DEFAULT_F64), (layout_dirty);
    height: f64, Auto(DEFAULT_F64), (layout_dirty);
    flex_basis: f64, Auto(DEFAULT_F64), (layout_dirty);
    flex_grow: f32, Absolute(0.), (layout_dirty);
    flex_shrink: f32, Absolute(0.), (layout_dirty);
    font_family: String, Absolute(String::from("sans-serif")), (layout_dirty, inherit);
    font_size: f32, Absolute(16.), (layout_dirty, inherit);
    line_height: f32, Auto(DEFAULT_F32), (layout_dirty, inherit);
    text_align: TextAlignType, Absolute(TextAlignType::Left), (layout_dirty, inherit);
    color: (f32, f32, f32, f32), Absolute((0., 0., 0., 1.)), (inherit);
    background_color: (f32, f32, f32, f32), Absolute((-1., -1., -1., -1.)), ();
    opacity: f32, Absolute(1.), ();
    transform: Transform, Absolute(Transform::new()), ();
    margin_left: f64, Absolute(0.), (layout_dirty);
    margin_right: f64, Absolute(0.), (layout_dirty);
    margin_top: f64, Absolute(0.), (layout_dirty);
    margin_bottom: f64, Absolute(0.), (layout_dirty);
    padding_left: f64, Absolute(0.), (layout_dirty);
    padding_right: f64, Absolute(0.), (layout_dirty);
    padding_top: f64, Absolute(0.), (layout_dirty);
    padding_bottom: f64, Absolute(0.), (layout_dirty);
    box_sizing: BoxSizingType, Absolute(BoxSizingType::ContentBox), (layout_dirty);
    border_left_width: f64, Absolute(0.), (layout_dirty);
    border_right_width: f64, Absolute(0.), (layout_dirty);
    border_top_width: f64, Absolute(0.), (layout_dirty);
    border_bottom_width: f64, Absolute(0.), (layout_dirty);
    border_left_style: BorderStyleType, Absolute(BorderStyleType::None), (layout_dirty);
    border_right_style: BorderStyleType, Absolute(BorderStyleType::None), (layout_dirty);
    border_top_style: BorderStyleType, Absolute(BorderStyleType::None), (layout_dirty);
    border_bottom_style: BorderStyleType, Absolute(BorderStyleType::None), (layout_dirty);
    border_left_color: (f32, f32, f32, f32), Absolute((-1., -1., -1., -1.)), ();
    border_right_color: (f32, f32, f32, f32), Absolute((-1., -1., -1., -1.)), ();
    border_top_color: (f32, f32, f32, f32), Absolute((-1., -1., -1., -1.)), ();
    border_bottom_color: (f32, f32, f32, f32), Absolute((-1., -1., -1., -1.)), ();
}

impl ElementStyle {
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
    fn element<'a>(&'a self) -> &'a Element {
        unsafe { &*self.element }
    }
    #[inline]
    fn element_mut<'a>(&'a mut self) -> &'a mut Element {
        unsafe { &mut *self.element }
    }
    #[inline]
    fn node<'a>(&'a self) -> &'a ForestNode<Element> {
        self.element().node()
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
        self.parent_updated();
    }
    #[inline]
    pub fn transform_ref(&mut self) -> &Transform {
        self.transform.get_value_ref()
    }
    #[inline]
    pub fn transform_mut(&mut self) -> &mut Transform {
        self.transform.get_value_mut()
    }
}
