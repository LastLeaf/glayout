use super::Element;

#[derive(Default)]
pub struct BoundingRect {
    dirty: bool,
    requested_width: f64,
    requested_height: f64,
    allocated_width: f64,
    allocated_height: f64,
    allocated_left: f64,
    allocated_top: f64,
}

impl BoundingRect {
    pub fn new() -> Self {
        BoundingRect {
            ..Default::default()
        }
    }
    pub fn mark_dirty(&mut self) {
        unimplemented!(); // TODO
    }
    pub fn recalculate(&mut self, elem: &Element) {
        unimplemented!(); // TODO
    }

    fn request_size(&mut self, elem: &Element) {
        unimplemented!(); // TODO
    }
    fn allocate_position(&mut self, elem: &Element) {
        unimplemented!(); // TODO
    }
}
