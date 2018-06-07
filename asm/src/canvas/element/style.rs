pub struct ElementStyle {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

impl ElementStyle {
    pub fn new() -> Self {
        ElementStyle {
            left: 0.,
            top: 0.,
            width: 0.,
            height: 0.,
        }
    }
}
