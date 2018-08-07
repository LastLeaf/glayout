use std::f64;

const DEFAULT_F64: f64 = f64::INFINITY;

pub struct ElementStyle {
    pub id: String,
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

#[derive(Clone, Copy, PartialEq)]
pub enum DisplayType {
    None,
    Block,
    Inline,
    InlineBlock,
    Flex,
}
