use std::f64;
use super::super::{Element, ElementStyle, DEFAULT_F64};
use super::{Point, Size, Position, Bounds, InlineAllocator, InlineAllocatorState, box_sizing};

// TODO

#[inline]
pub fn get_min_max_width(element: &mut Element, style: &ElementStyle, inline_allocator: &mut InlineAllocator) -> (f64, f64) {
    let (_margin, _border, _padding, content) = box_sizing::get_sizes(style, Size::new(DEFAULT_F64, DEFAULT_F64));

    let min_max_width = if element.is_terminated() {
        element.content_mut().suggest_size(Size::new(f64::MAX, 0.), inline_allocator, style);
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

    min_max_width
}

#[inline]
pub fn suggest_size(element: &mut Element, style: &ElementStyle, suggested_size: Size, inline_allocator: &mut InlineAllocator) -> Size {
    // for inline nodes
    // the returned width is the current end width of the inline allocation
    // the returned height is the "added" height related to prev sibling

    let child_suggested_size = Size::new(suggested_size.width(), 0.);
    let prev_filled_height = inline_allocator.get_current_filled_height();
    let mut child_inline_allocator = InlineAllocator::new();
    child_inline_allocator.reset(element.node_mut(), &InlineAllocatorState::new(child_suggested_size.width(), style.get_text_align()));

    let mut child_requested_height = 0.;
    if element.is_terminated() {
        element.content_mut().suggest_size(child_suggested_size, &mut child_inline_allocator, style);
        child_requested_height = child_inline_allocator.get_current_height();
    } else {
        let node = element.node_mut();
        node.for_each_child_mut(|child| {
            let size = child.suggest_size(child_suggested_size, &mut child_inline_allocator, true);
            child_requested_height += size.height();
        });
    }

    let baseline_offset = child_inline_allocator.get_current_height() / 2.; // FIXME impl vertical-align
    let (left, inline_baseline_top) = {
        let node = element.node_mut();
        child_inline_allocator.end(node);
        inline_allocator.start_node(node, child_inline_allocator.get_current_height(), baseline_offset);
        inline_allocator.add_width(node, child_inline_allocator.get_current_width(), true)
    }.into();
    element.position_offset.inline_position_offset = Size::new(left, inline_baseline_top - baseline_offset);

    Size::new(child_inline_allocator.get_current_width(), child_requested_height)
}

#[inline]
pub fn allocate_position(element: &mut Element, _style: &ElementStyle, allocated_point: Point, relative_point: Point) -> (Point, Bounds) {
    let requested_size = element.requested_size();
    let mut drawing_bounds = Bounds::new(0., 0., requested_size.width(), requested_size.height());
    if element.content().is_terminated() {
        drawing_bounds.union(&element.content().drawing_bounds());
    } else {
        let mut current_top = 0.;
        let node = element.node_mut();
        node.for_each_child_mut(|child| {
            let requested_size = child.requested_size();
            let child_bounds = child.allocate_position(
                Point::new(0., current_top),
                relative_point + Size::new(0., -current_top)
            );
            drawing_bounds.union(&child_bounds);
            if !box_sizing::is_independent_positioning(child.style()) {
                current_top += requested_size.height();
            }
        });
    }
    drawing_bounds.move_size(element.position_offset.inline_position_offset);
    (allocated_point + element.position_offset.inline_position_offset, drawing_bounds)
}
