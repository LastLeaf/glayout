use std::rc::Rc;
use super::super::CanvasConfig;
use super::{ElementStyle, Transform};

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
    fn clone(&self) -> Box<super::ElementContent> {
        Box::new(Empty {})
    }
    #[inline]
    fn draw(&mut self, _style: &ElementStyle, _transform: &Transform) {
        // do nothing
        // debug!("Attempted to draw an Empty");
    }
    fn drawing_bounds(&self) -> (f64, f64, f64, f64) {
        (0., 0., 0., 0.) // not used because it is not a terminated
    }
    fn is_under_point(&self, _x: f64, _y: f64, _transform: Transform) -> bool {
        false // not used because it is not a terminated
    }
}
