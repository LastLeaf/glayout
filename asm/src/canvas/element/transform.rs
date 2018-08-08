#[derive(Clone, Copy, PartialEq)]
pub struct Transform {
    pub offset: (f64, f64),
}

impl Transform {
    pub fn new() -> Self {
        Self {
            offset: (0., 0.),
        }
    }
}
