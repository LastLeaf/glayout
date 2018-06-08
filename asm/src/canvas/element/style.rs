#[derive(Default)]
pub struct ElementStyle {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

impl ElementStyle {
    pub fn new() -> Self {
        ElementStyle { ..Default::default() }
    }
}
