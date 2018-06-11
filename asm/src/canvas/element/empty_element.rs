use super::super::CanvasConfig;
use super::{ElementStyle, BoundingRect};

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
    fn draw(&mut self, _style: &ElementStyle, _bounding_rect: &BoundingRect) {
        // do nothing
        // debug!("Attempted to draw an EmptyElement");
    }
}
