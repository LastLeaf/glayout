#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    x: (f64, f64, f64, f64),
    y: (f64, f64, f64, f64),
    z: (f64, f64, f64, f64),
}

impl Transform {
    pub fn new() -> Self {
        Self {
            x: (1., 0., 0., 0.),
            y: (0., 1., 0., 0.),
            z: (0., 0., 1., 0.),
        }
    }
    pub fn reset(&mut self) -> &mut Self {
        self.x = (1., 0., 0., 0.);
        self.y = (0., 1., 0., 0.);
        self.z = (0., 0., 1., 0.);
        self
    }
    pub fn offset(&mut self, left: f64, top: f64) -> &mut Self {
        self.x.3 += left;
        self.y.3 += top;
        self
    }
    #[inline]
    pub fn get_offset(&self) -> (f64, f64) {
        (self.x.3, self.y.3)
    }
    pub fn reset_scale(&mut self, scale_x: f64, scale_y: f64) -> &mut Self {
        self.x.0 = scale_x;
        self.y.1 = scale_y;
        self
    }
    pub fn scale(&mut self, scale_x: f64, scale_y: f64) -> &mut Self {
        self.x.0 *= scale_x;
        self.y.1 *= scale_y;
        self
    }
    #[inline]
    pub fn get_scale(&self) -> (f64, f64) {
        (self.x.0, self.y.1)
    }
    pub fn mul_clone(&mut self, t: &Self) -> Self {
        let new_x = (
            self.x.0 * t.x.0 + self.x.1 * t.y.0 + self.x.2 * t.z.0,
            self.x.0 * t.x.1 + self.x.1 * t.y.1 + self.x.2 * t.z.1,
            self.x.0 * t.x.2 + self.x.1 * t.y.2 + self.x.2 * t.z.2,
            self.x.0 * t.x.3 + self.x.1 * t.y.3 + self.x.2 * t.z.3 + self.x.3,
        );
        let new_y = (
            self.y.0 * t.x.0 + self.y.1 * t.y.0 + self.y.2 * t.z.0,
            self.y.0 * t.x.1 + self.y.1 * t.y.1 + self.y.2 * t.z.1,
            self.y.0 * t.x.2 + self.y.1 * t.y.2 + self.y.2 * t.z.2,
            self.y.0 * t.x.3 + self.y.1 * t.y.3 + self.y.2 * t.z.3 + self.y.3,
        );
        let new_z = (
            self.z.0 * t.x.0 + self.z.1 * t.y.0 + self.z.2 * t.z.0,
            self.z.0 * t.x.1 + self.z.1 * t.y.1 + self.z.2 * t.z.1,
            self.z.0 * t.x.2 + self.z.1 * t.y.2 + self.z.2 * t.z.2,
            self.z.0 * t.x.3 + self.z.1 * t.y.3 + self.z.2 * t.z.3 + self.z.3,
        );
        Self {
            x: new_x,
            y: new_y,
            z: new_z,
        }
    }
    #[inline]
    pub fn apply_to_point(&self, pointer: (f64, f64)) -> (f64, f64) {
        (
            self.x.0 * pointer.0 + self.x.1 * pointer.1 + self.x.3,
            self.y.0 * pointer.0 + self.y.1 * pointer.1 + self.y.3,
        )
    }
    #[inline]
    pub fn apply_to_position(&self, pos: &(f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
        let (x, y) = self.apply_to_point((pos.0, pos.1));
        let (xw, yh) = self.apply_to_point((pos.0 + pos.2, pos.1 + pos.3));
        (x, y, xw - x, yh - y)
    }
    #[inline]
    pub fn apply_to_bounds(&self, pos: &(f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
        let (x, y) = self.apply_to_point((pos.0, pos.1));
        let (xw, yh) = self.apply_to_point((pos.2, pos.3));
        (x, y, xw, yh)
    }
}
