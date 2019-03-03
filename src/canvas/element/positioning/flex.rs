use super::super::{Element, ElementStyle, DEFAULT_F64};
use super::{Point, Size, Position, Bounds, InlineAllocator, box_sizing};

#[inline]
pub fn suggest_size(element: &mut Element, style: &ElementStyle, suggested_size: Size, inline_allocator: &mut InlineAllocator) -> Size {
    let (margin, _border, _padding, content) = box_sizing::get_sizes(style, Size::new(suggested_size.width(), DEFAULT_F64));
    let child_suggested_size = content;

    let node = element.node_mut();
    let child_min_max_basis = Vec::with_capacity(node.len());
    let mut min_total = 0.;
    let mut basis_total = 0.;
    node.for_each_child_mut(|child| {
        let (min, max) = child.position_offset.get_min_max_width(&mut InlineAllocator::new());
        let basis = max;
        child_min_max_basis.push((min, max, basis));
        min_total += min;
        basis_total += basis;
    });
    if basis_total < suggested_size.width() {

    }

    let node = element.node_mut();
    node.for_each_child_mut(|child| {
        let size = child.suggest_size(child_suggested_size, inline_allocator, false);
        child_requested_height += size.height();
    });

    if style.get_height() == DEFAULT_F64 {
        margin + Size::new(0., child_requested_height)
    } else {
        margin
    }
}

#[inline]
pub fn allocate_position(element: &mut Element, style: &ElementStyle, allocated_point: Point, relative_point: Point) -> (Point, Bounds) {
    let (_margin, _border, _padding, content) = box_sizing::get_offsets(style);
    let requested_size = element.requested_size();
    let allocated_position = Position::from((allocated_point, requested_size));
    let mut drawing_bounds: Bounds = allocated_position.into();
    if element.content().is_terminated() {
        drawing_bounds.union(&element.content().drawing_bounds());
    } else {
        let mut current_top = content.top();
        let node = element.node_mut();
        node.for_each_child_mut(|child| {
            let requested_size = child.requested_size();
            let child_bounds = child.allocate_position(
                Point::new(content.left(), current_top),
                relative_point + Size::new(-content.left(), -current_top)
            ) + content.into();
            drawing_bounds.union(&child_bounds);
            if !box_sizing::is_independent_positioning(child.style()) {
                current_top += requested_size.height();
            }
        });
    }
    (allocated_point, drawing_bounds)
}
