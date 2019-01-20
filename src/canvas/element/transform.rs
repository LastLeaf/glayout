use super::{Position, Size, Point, Bounds};

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
    pub fn offset(&mut self, s: Size) -> &mut Self {
        self.x.3 += s.width();
        self.y.3 += s.height();
        self
    }
    #[inline]
    pub fn get_offset(&self) -> Size {
        Size::new(self.x.3, self.y.3)
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
    pub fn apply_to_point(&self, pointer: Point) -> Point {
        Point::new(
            self.x.0 * pointer.left() + self.x.1 * pointer.top() + self.x.3,
            self.y.0 * pointer.left() + self.y.1 * pointer.top() + self.y.3,
        )
    }
    #[inline]
    pub fn apply_to_position(&self, pos: &Position) -> Position {
        let (x, y) = self.apply_to_point(Point::new(pos.left(), pos.top())).into();
        let (xw, yh) = self.apply_to_point(Point::new(pos.right(), pos.bottom())).into();
        Position::new(x, y, xw - x, yh - y)
    }
    #[inline]
    pub fn apply_to_bounds(&self, pos: &Bounds) -> Bounds {
        let (x, y) = self.apply_to_point(Point::new(pos.left(), pos.top())).into();
        let (xw, yh) = self.apply_to_point(Point::new(pos.right(), pos.bottom())).into();
        Bounds::new(x, y, xw, yh)
    }
}
