use std::ops::{Add, Sub, Neg};

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Point {
    left: f64,
    top: f64,
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Size {
    width: f64,
    height: f64,
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Position {
    point: Point,
    size: Size,
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Bounds {
    point1: Point,
    point2: Point,
}

impl Point {
    #[inline]
    pub fn new(left: f64, top: f64) -> Self {
        Self {
            left,
            top,
        }
    }
    #[inline]
    pub fn left(&self) -> f64 {
        self.left
    }
    #[inline]
    pub fn top(&self) -> f64 {
        self.top
    }
    #[inline]
    pub fn move_size(&mut self, s: Size) {
        self.left += s.width;
        self.top += s.height;
    }
    pub fn in_position(&self, pos: &Position) -> bool {
        if self.left < pos.left() { return false }
        if self.top < pos.top() { return false }
        if self.left >= pos.right() { return false }
        if self.top >= pos.bottom() { return false }
        true
    }
    pub fn in_bounds(&self, pos: &Bounds) -> bool {
        if self.left < pos.left() { return false }
        if self.top < pos.top() { return false }
        if self.left >= pos.right() { return false }
        if self.top >= pos.bottom() { return false }
        true
    }
}

impl Add<Size> for Point {
    type Output = Point;

    fn add(self, other: Size) -> Point {
        Point {
            left: self.left + other.width,
            top: self.top + other.height,
        }
    }
}

impl Sub<Point> for Point {
    type Output = Size;

    fn sub(self, other: Point) -> Size {
        Size {
            width: self.left - other.left,
            height: self.top - other.top,
        }
    }
}

impl Into<Size> for Point {
    fn into(self) -> Size {
        Size::new(self.left, self.top)
    }
}

impl Into<(f64, f64)> for Point {
    fn into(self) -> (f64, f64) {
        (self.left, self.top)
    }
}

impl Size {
    #[inline]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
        }
    }
    #[inline]
    pub fn width(&self) -> f64 {
        self.width
    }
    #[inline]
    pub fn height(&self) -> f64 {
        self.height
    }
    #[inline]
    pub fn add_size(&mut self, s: Size) {
        self.width += s.width;
        self.height += s.height;
    }
    #[inline]
    pub fn add_width(&mut self, width: f64) {
        self.width += width;
    }
    #[inline]
    pub fn add_height(&mut self, height: f64) {
        self.height += height;
    }
}

impl Add<Size> for Size {
    type Output = Size;

    fn add(self, other: Size) -> Size {
        Size {
            width: self.width + other.width,
            height: self.height + other.height,
        }
    }
}

impl Sub<Size> for Size {
    type Output = Size;

    fn sub(self, other: Size) -> Size {
        Size {
            width: self.width - other.width,
            height: self.height - other.height,
        }
    }
}

impl Neg for Size {
    type Output = Size;

    fn neg(self) -> Size {
        Size {
            width: - self.width,
            height: - self.height,
        }
    }
}

impl Into<(f64, f64)> for Size {
    fn into(self) -> (f64, f64) {
        (self.width, self.height)
    }
}

impl Position {
    #[inline]
    pub fn new(left: f64, top: f64, width: f64, height: f64) -> Self {
        Self {
            point: Point::new(left, top),
            size: Size::new(width, height),
        }
    }
    #[inline]
    pub fn left(&self) -> f64 {
        self.point.left()
    }
    #[inline]
    pub fn top(&self) -> f64 {
        self.point.top()
    }
    #[inline]
    pub fn width(&self) -> f64 {
        self.size.width()
    }
    #[inline]
    pub fn height(&self) -> f64 {
        self.size.height()
    }
    #[inline]
    pub fn right(&self) -> f64 {
        self.point.left() + self.size.width()
    }
    #[inline]
    pub fn bottom(&self) -> f64 {
        self.point.top() + self.size.height()
    }
    #[inline]
    pub fn left_top(&self) -> Point {
        self.point
    }
    pub fn move_size(&mut self, s: Size) {
        self.point.move_size(s);
    }
    pub fn shrink(&mut self, left_top: Size, right_bottom: Size) {
        self.point.move_size(left_top);
        self.size.add_size(- left_top - right_bottom);
    }
}

impl Into<(f64, f64, f64, f64)> for Position {
    fn into(self) -> (f64, f64, f64, f64) {
        (self.point.left, self.point.top, self.size.width, self.size.height)
    }
}

impl From<(Point, Size)> for Position {
    fn from(other: (Point, Size)) -> Position {
        Position {
            point: other.0,
            size: other.1,
        }
    }
}

impl From<Bounds> for Position {
    fn from(other: Bounds) -> Position {
        Position {
            point: other.point1,
            size: other.point2 - other.point1,
        }
    }
}

impl Bounds {
    pub fn new(left: f64, top: f64, right: f64, bottom: f64) -> Self {
        Self {
            point1: Point::new(left, top),
            point2: Point::new(right, bottom),
        }
    }
    #[inline]
    pub fn left(&self) -> f64 {
        self.point1.left()
    }
    #[inline]
    pub fn top(&self) -> f64 {
        self.point1.top()
    }
    #[inline]
    pub fn width(&self) -> f64 {
        self.point2.left() - self.point1.left()
    }
    #[inline]
    pub fn height(&self) -> f64 {
        self.point2.top() - self.point1.top()
    }
    #[inline]
    pub fn right(&self) -> f64 {
        self.point2.left()
    }
    #[inline]
    pub fn bottom(&self) -> f64 {
        self.point2.top()
    }
    pub fn extend_left(&mut self, v: f64) {
        self.point1.left += v;
    }
    pub fn extend_top(&mut self, v: f64) {
        self.point1.top += v;
    }
    pub fn extend_right(&mut self, v: f64) {
        self.point2.left += v;
    }
    pub fn extend_bottom(&mut self, v: f64) {
        self.point2.top += v;
    }
    pub fn union(&mut self, other: &Self) {
        if self.point1.left > other.point1.left { self.point1.left = other.point1.left };
        if self.point1.top > other.point1.top { self.point1.top = other.point1.top };
        if self.point2.left < other.point2.left { self.point2.left = other.point2.left };
        if self.point2.top < other.point2.top { self.point2.top = other.point2.top };
    }
    pub fn intersection(&mut self, other: &Self) {
        if self.point1.left < other.point1.left { self.point1.left = other.point1.left };
        if self.point1.top < other.point1.top { self.point1.top = other.point1.top };
        if self.point2.left > other.point2.left { self.point2.left = other.point2.left };
        if self.point2.top > other.point2.top { self.point2.top = other.point2.top };
    }
}

impl Add<Size> for Bounds {
    type Output = Bounds;

    fn add(self, other: Size) -> Bounds {
        Bounds {
            point1: self.point1 + other,
            point2: self.point2 + other,
        }
    }
}

impl From<Position> for Bounds {
    fn from(other: Position) -> Bounds {
        Bounds {
            point1: other.point,
            point2: other.point + other.size
        }
    }
}
