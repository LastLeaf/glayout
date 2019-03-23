use std::f64;
use super::super::{Element, ElementStyle};
use super::{Point, Size, Position, Bounds, InlineAllocator, InlineAllocatorState, box_sizing};

#[inline]
pub fn get_min_max_width(element: &mut Element, style: &ElementStyle, inline_allocator: &mut InlineAllocator) -> (f64, f64) {
    let (_margin, _border, _padding, content) = box_sizing::get_sizes(style, Size::new(f64::INFINITY, f64::INFINITY));

    inline_allocator.reset_with_current_state(element.node_mut());
    let min_max_width = if element.is_terminated() {
        element.content_mut().suggest_size(Size::new(f64::MAX, f64::INFINITY), inline_allocator, style);
        inline_allocator.get_min_max_width()
    } else {
        let node = element.node_mut();
        let mut min_width = 0.;
        let mut max_width = 0.;
        node.for_each_child_mut(|child| {
            let (min, max) = child.position_offset.get_min_max_width(inline_allocator);
            if min_width < min { min_width = min };
            if max_width < max { max_width = max };
        });
        (min_width, max_width)
    };
    inline_allocator.reset_with_current_state(element.node_mut());

    min_max_width
}

#[inline]
pub fn suggest_size(element: &mut Element, style: &ElementStyle, suggested_size: Size, inline_allocator: &mut InlineAllocator) -> Size {
    let (margin, _border, _padding, content) = box_sizing::get_sizes(style, Size::new(suggested_size.width(), f64::INFINITY));
    let child_suggested_size = Size::new(
        content.width(),
        if style.get_height().is_finite() {
            content.height()
        } else {
            f64::INFINITY
        }
    );

    let state = inline_allocator.state().clone();
    inline_allocator.reset(element.node_mut(), &InlineAllocatorState::new(content.width(), style.get_text_align()));
    let mut child_requested_height = 0.;
    if element.is_terminated() {
        let _size = element.content_mut().suggest_size(child_suggested_size, inline_allocator, style);
        child_requested_height = inline_allocator.get_current_height();
    } else {
        let node = element.node_mut();
        node.for_each_child_mut(|child| {
            let size = child.position_offset.suggest_size(child_suggested_size, inline_allocator, true);
            child_requested_height += size.height();
        });
    }
    inline_allocator.reset(element.node_mut(), &state);

    if !style.get_height().is_finite() {
        margin + Size::new(0., child_requested_height)
    } else {
        margin
    }
}

#[inline]
pub fn allocate_position(element: &mut Element, style: &ElementStyle, allocated_point: Point, relative_point: Point) -> (Point, Bounds) {
    let suggested_size = element.position_offset.suggested_size;
    let requested_size = element.position_offset.requested_size;
    let (_margin, _border, _padding, content) = box_sizing::get_offsets(style, suggested_size, requested_size);
    let allocated_position = Position::from((allocated_point, requested_size));
    let mut drawing_bounds: Bounds = allocated_position.into();
    if element.content().is_terminated() {
        drawing_bounds.union(&element.content().drawing_bounds());
    } else {
        let mut current_top = content.top();
        let node = element.node_mut();
        node.for_each_child_mut(|child| {
            let requested_size = child.position_offset.requested_size();
            let child_bounds = child.position_offset.allocate_position(
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
