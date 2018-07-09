use std::rc::Rc;
use super::super::CanvasConfig;
use super::{ElementStyle, PositionOffset};

#[derive(Debug)]
pub struct Empty {}

impl Empty {
    pub fn new(_cfg: &Rc<CanvasConfig>) -> Self {
        Empty {}
    }
}

impl super::ElementContent for Empty {
    #[inline]
    fn name(&self) -> &'static str {
        "Empty"
    }
    fn draw(&mut self, _style: &ElementStyle, _bounding_rect: &PositionOffset) {
        // do nothing
        // debug!("Attempted to draw an Empty");
    }
}
