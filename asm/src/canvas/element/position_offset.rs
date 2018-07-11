use super::style::{DisplayType};
use super::Element;

#[derive(Default, Debug)]
pub struct PositionOffset {
    suggested_size: (f64, f64),
    requested_size: (f64, f64),
    allocated_position: (f64, f64, f64, f64),
}

impl PositionOffset {
    pub fn new() -> Self {
        PositionOffset {
            ..Default::default()
        }
    }

    #[inline]
    pub fn get_requested_size(&self) -> (f64, f64) {
        self.requested_size
    }
    #[inline]
    pub fn get_allocated_position(&self) -> (f64, f64, f64, f64) {
        self.allocated_position
    }

    pub fn suggest_size(&mut self, is_dirty: bool, suggested_size: (f64, f64), element: &Element) -> (f64, f64) {
        if !is_dirty && suggested_size == self.suggested_size {
            return self.requested_size
        }
        self.suggested_size = suggested_size;
        let mut request_width = 0.;
        let mut request_height = 0.;
        let style = element.style();
        match style.display {
            DisplayType::Block => {
                request_width = suggested_size.0;
                if element.content().is_terminated() {
                    let (_, h) = element.content_mut().suggest_size((suggested_size.0, 0.), &*element.style());
                    request_height += h;
                } else {
                    for child in element.tree_node().iter_children() {
                        let (_, h) = child.elem().suggest_size((suggested_size.0, 0.));
                        request_height += h;
                    }
                }
            },
            _ => {
                unimplemented!();
            }
        };
        self.requested_size = (request_width, request_height);
        self.requested_size
    }
    pub fn allocate_position(&mut self, is_dirty: bool, allocated_position: (f64, f64, f64, f64), element: &Element) {
        if !is_dirty && allocated_position == self.allocated_position {
            return
        }
        let style = element.style();
        match style.display {
            DisplayType::Block => {
                self.allocated_position = allocated_position;
                let mut current_height = 0.;
                if element.content().is_terminated() {
                    /* do nothing */
                } else {
                    for child in element.tree_node().iter_children() {
                        let element = child.elem();
                        let (_, h) = element.get_requested_size();
                        element.allocate_position((0., current_height, allocated_position.2, h));
                        current_height += h;
                    }
                }
            },
            _ => {
                unimplemented!();
            }
        };
    }
}


