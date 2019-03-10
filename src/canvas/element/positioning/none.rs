use super::super::{Element, ElementStyle};
use super::{Size, Point, Bounds, InlineAllocator};

#[inline]
pub fn get_min_max_width(_element: &mut Element, _style: &ElementStyle, _inline_allocator: &mut InlineAllocator) -> (f64, f64) {
    (0., 0.)
}

#[inline]
pub fn suggest_size(_element: &Element, _style: &ElementStyle, _suggested_size: Size, _inline_allocator: &mut InlineAllocator) -> Size {
    Size::new(0., 0.)
}

#[inline]
pub fn allocate_position(_element: &Element, _style: &ElementStyle, allocated_point: Point, _relative_point: Point) -> (Point, Bounds) {
    (
        allocated_point,
        Bounds::new(allocated_point.left(), allocated_point.top(), allocated_point.left(), allocated_point.top())
    )
}
