use std::cell::Cell;
use std::rc::Rc;
use std::borrow::Cow;
use std::{f32, f64};
use std::any::Any;
use super::{Element, Transform};
use rc_forest::{ForestNode};
use glayout_element_style_macro::*;

mod helper_macros;
mod types;
pub use self::types::*;
mod class;
pub use self::class::ElementClass;
mod style_sheet;
pub use self::style_sheet::{StyleSheetGroup, StyleSheet};
mod style_value;
pub(crate) use self::style_value::{StyleValue, StyleValueReferrer};

const DEFAULT_FONT_SIZE: f32 = 16.;
const DEFAULT_F64: f64 = f64::INFINITY;
const DEFAULT_F32: f32 = f32::INFINITY;

element_style! {
    display: DisplayType, Absolute(DisplayType::Inline), 0x02, (layout_dirty);
    opacity: f32, Absolute(1.), 0x03, ();
    transform: Transform, Absolute(Transform::new()), 0x04, ();

    box_sizing: BoxSizingType, Absolute(BoxSizingType::ContentBox), 0x08, (layout_dirty);
    width: f64, Auto(DEFAULT_F64), 0x09, (layout_dirty, horizontal_relative);
    height: f64, Auto(DEFAULT_F64), 0x0a, (layout_dirty, vertical_relative);

    position: PositionType, Absolute(PositionType::Static), 0x10, (layout_dirty);
    left: f64, Auto(DEFAULT_F64), 0x11, (layout_dirty, horizontal_relative);
    right: f64, Auto(DEFAULT_F64), 0x12, (layout_dirty, horizontal_relative);
    top: f64, Auto(DEFAULT_F64), 0x13, (layout_dirty, vertical_relative);
    bottom: f64, Auto(DEFAULT_F64), 0x14, (layout_dirty, vertical_relative);

    flex_direction: FlexDirectionType, Absolute(FlexDirectionType::Row), 0x20, (layout_dirty);
    flex_grow: f32, Absolute(0.), 0x21, (layout_dirty);
    flex_shrink: f32, Absolute(0.), 0x22, (layout_dirty);
    flex_basis: f64, Auto(DEFAULT_F64), 0x23, (layout_dirty, flex_direction_relative);

    font_family: Cow<'static, str>, Absolute(Cow::from(String::from("sans-serif"))), 0x30, (layout_dirty, inherit);
    font_size: f32, Absolute(DEFAULT_FONT_SIZE), 0x31, (layout_dirty, inherit, font_size_relative, font_size_inherit);
    line_height: f32, Auto(DEFAULT_F32), 0x32, (layout_dirty, inherit, font_size_relative);
    text_align: TextAlignType, Absolute(TextAlignType::Left), 0x33, (layout_dirty, inherit);
    color: (f32, f32, f32, f32), Absolute((0., 0., 0., 1.)), 0x34, (inherit);
    background_color: (f32, f32, f32, f32), Absolute((-1., -1., -1., -1.)), 0x35, ();

    margin_left: f64, Absolute(0.), 0x40, (layout_dirty, horizontal_relative);
    margin_right: f64, Absolute(0.), 0x41, (layout_dirty, horizontal_relative);
    margin_top: f64, Absolute(0.), 0x42, (layout_dirty, vertical_relative);
    margin_bottom: f64, Absolute(0.), 0x43, (layout_dirty, vertical_relative);
    padding_left: f64, Absolute(0.), 0x44, (layout_dirty, horizontal_relative);
    padding_right: f64, Absolute(0.), 0x45, (layout_dirty, horizontal_relative);
    padding_top: f64, Absolute(0.), 0x46, (layout_dirty, vertical_relative);
    padding_bottom: f64, Absolute(0.), 0x47, (layout_dirty, vertical_relative);

    border_left_width: f64, Absolute(0.), 0x50, (layout_dirty, horizontal_relative);
    border_right_width: f64, Absolute(0.), 0x51, (layout_dirty, horizontal_relative);
    border_top_width: f64, Absolute(0.), 0x52, (layout_dirty, vertical_relative);
    border_bottom_width: f64, Absolute(0.), 0x53, (layout_dirty, vertical_relative);
    border_left_style: BorderStyleType, Absolute(BorderStyleType::None), 0x54, (layout_dirty);
    border_right_style: BorderStyleType, Absolute(BorderStyleType::None), 0x55, (layout_dirty);
    border_top_style: BorderStyleType, Absolute(BorderStyleType::None), 0x56, (layout_dirty);
    border_bottom_style: BorderStyleType, Absolute(BorderStyleType::None), 0x57, (layout_dirty);
    border_left_color: (f32, f32, f32, f32), Absolute((-1., -1., -1., -1.)), 0x58, ();
    border_right_color: (f32, f32, f32, f32), Absolute((-1., -1., -1., -1.)), 0x59, ();
    border_top_color: (f32, f32, f32, f32), Absolute((-1., -1., -1., -1.)), 0x5a, ();
    border_bottom_color: (f32, f32, f32, f32), Absolute((-1., -1., -1., -1.)), 0x5b, ();
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
    pub(super) unsafe fn _clone_mut_unsafe<'a, 'b>(&'a mut self) -> &'b mut Self {
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
