pub struct ElementStyle {
    pub id: String,
    pub display: DisplayType,
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub font_family: String,
    pub font_size: f64,
}

impl ElementStyle {
    pub fn new() -> Self {
        ElementStyle {
            id: String::new(),
            display: DisplayType::Inline,
            left: 0.,
            top: 0.,
            width: 0.,
            height: 0.,
            font_family: String::from("sans-serif"),
            font_size: 16.,
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
