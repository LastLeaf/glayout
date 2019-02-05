use super::super::{Element, ElementStyle};
use super::{Size, Point, Bounds, InlineAllocator};

#[inline]
pub fn suggest_size(_element: &Element, _style: &ElementStyle, _suggested_size: Size, _inline_allocator: &mut InlineAllocator, _handle_absolute: bool) -> Size {
    Size::new(0., 0.)
}

#[inline]
pub fn allocate_position(_element: &Element, _style: &ElementStyle, allocated_point: Point, _relative_point: Point) -> Bounds {
    Bounds::new(allocated_point.left(), allocated_point.top(), allocated_point.left(), allocated_point.top())
}
