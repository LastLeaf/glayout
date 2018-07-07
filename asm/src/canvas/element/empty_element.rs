use std::rc::Rc;
use super::super::CanvasConfig;
use super::{ElementStyle, PositionOffset};

#[derive(Debug)]
pub struct EmptyElement {}

impl EmptyElement {
    pub fn new(_cfg: &Rc<CanvasConfig>) -> Self {
        EmptyElement {}
    }
}

impl super::ElementContent for EmptyElement {
    #[inline]
    fn name(&self) -> &'static str {
        "EmptyElement"
    }
    fn draw(&mut self, _style: &ElementStyle, _bounding_rect: &PositionOffset) {
        // do nothing
        // debug!("Attempted to draw an EmptyElement");
    }
}
