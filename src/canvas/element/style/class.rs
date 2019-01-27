use std::any::Any;
use std::slice::Iter;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StyleName {
    glayout_unrecognized = 0x00,
    display = 0x02,

    position = 0x10,
    left = 0x11,
    top = 0x12,
    width = 0x13,
    height = 0x14,

    font_family = 0x20,
    font_size = 0x21,
    line_height = 0x22,
    text_align = 0x23,

    color = 0x30,
    background_color = 0x31,
    opacity = 0x32,

    transform = 0x40,

    margin_left = 0x50,
    margin_right = 0x51,
    margin_top = 0x52,
    margin_bottom = 0x53,
    padding_left = 0x54,
    padding_right = 0x55,
    padding_top = 0x56,
    padding_bottom = 0x57,
}

#[derive(Default)]
pub struct ElementClass {
    rules: Vec<(StyleName, Box<Any + Send>)>,
}

impl ElementClass {
    pub fn new() -> Self {
        Self {
            rules: vec![]
        }
    }
    pub fn add_rule(&mut self, name: StyleName, value: Box<Any + Send>) {
        self.rules.push((name, value))
    }
    pub fn replace_rule(&mut self, name: StyleName, value: Box<Any + Send>) {
        let p = self.rules.iter().position(|x| x.0 == name);
        match p {
            Some(p) => {
                self.rules.remove(p);
            },
            None => { }
        }
        self.add_rule(name, value)
    }
    pub fn iter_rules(&self) -> Iter<(StyleName, Box<Any + Send>)> {
        self.rules.iter()
    }
    pub fn from_style_text(&mut self, text: &str) {
        self.rules.truncate(0);
        super::StyleSheet::parse_inline_style(self, text);
    }
    pub fn apply_to_style(&self, style: &mut super::ElementStyle) {
        for (name, value) in self.rules.iter() {
            self.apply_rule(style, name, value);
        }
    }

    fn apply_rule(&self, style: &mut super::ElementStyle, name: &StyleName, value: &Box<Any + Send>) {
        macro_rules! style_name {
            ($field: ident, $type: ty) => {
                {
                    style.$field(value.downcast_ref::<$type>().unwrap().clone());
                }
            }
        }
        match name {
            StyleName::glayout_unrecognized => { },
            StyleName::display => style_name!(display, super::DisplayType),
            StyleName::position => style_name!(position, super::PositionType),
            StyleName::left => style_name!(left, f64),
            StyleName::top => style_name!(top, f64),
            StyleName::width => style_name!(width, f64),
            StyleName::height => style_name!(height, f64),
            StyleName::font_family => style_name!(font_family, String),
            StyleName::font_size => style_name!(font_size, f32),
            StyleName::line_height => style_name!(line_height, f32),
            StyleName::text_align => style_name!(text_align, super::TextAlignType),
            StyleName::color => style_name!(color, (f32, f32, f32, f32)),
            StyleName::background_color => style_name!(background_color, (f32, f32, f32, f32)),
            StyleName::opacity => style_name!(opacity, f32),
            StyleName::transform => style_name!(transform, super::Transform),
            StyleName::margin_left => style_name!(margin_left, f64),
            StyleName::margin_right => style_name!(margin_right, f64),
            StyleName::margin_top => style_name!(margin_top, f64),
            StyleName::margin_bottom => style_name!(margin_bottom, f64),
            StyleName::padding_left => style_name!(padding_left, f64),
            StyleName::padding_right => style_name!(padding_right, f64),
            StyleName::padding_top => style_name!(padding_top, f64),
            StyleName::padding_bottom => style_name!(padding_bottom, f64),
        }
    }
}
