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
    #[inline]
    fn is_terminated(&self) -> bool {
        false
    }
    #[inline]
    fn draw(&mut self, _style: &ElementStyle, _pos: (f64, f64, f64, f64)) {
        // do nothing
        // debug!("Attempted to draw an Empty");
    }
}
