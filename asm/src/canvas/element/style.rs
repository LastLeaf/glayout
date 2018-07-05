#[derive(Default)]
pub struct ElementStyle {
    pub id: &'static str,
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub font_family: String,
    pub font_size: f64,
}

impl ElementStyle {
    pub fn new() -> Self {
        ElementStyle { ..Default::default() }
    }
}
