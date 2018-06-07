use super::super::CanvasConfig;
use super::ElementStyle;

#[derive(Debug)]
pub struct EmptyElement {}

impl EmptyElement {
    pub fn new(_cfg: &CanvasConfig) -> Self {
        EmptyElement {}
    }
}

impl super::ElementContent for EmptyElement {
    fn name(&self) -> &'static str {
        "EmptyElement"
    }
    fn draw(&self, _elem: &ElementStyle) {
        // do nothing
        // println!("Attempted to draw an EmptyElement");
    }
}
