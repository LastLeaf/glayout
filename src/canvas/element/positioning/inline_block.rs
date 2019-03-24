use std::f64;
use super::super::{Element, ElementStyle};
use super::{Point, Size, Bounds, InlineAllocator, InlineAllocatorState, box_sizing};

#[inline]
pub fn get_min_max_width(element: &mut Element, style: &ElementStyle, inline_allocator: &mut InlineAllocator) -> (f64, f64) {
    let (offset, non_auto_width) = box_sizing::get_h_offset(style);

    let min_max_width = if non_auto_width {
        (style.get_width(), style.get_width())
    } else if element.is_terminated() {
        element.content_mut().suggest_size(Size::new(f64::MAX, f64::NAN), inline_allocator, style);
        let (min, max) = inline_allocator.get_min_max_width();
        (min + offset, max + offset)
    } else {
        // TODO impl a dedicated get_min_max_width to solve
        let node = element.node_mut();
        let mut min_width = 0.;
        let mut max_width = 0.;
        node.for_each_child_mut(|child| {
            let (min, max) = child.position_offset.min_max_width_dfs(inline_allocator, false);
            if min_width < min { min_width = min };
            if max_width < max { max_width = max };
        });
        (min_width + offset, max_width + offset)
    };

    min_max_width
}

#[inline]
pub fn suggest_size(element: &mut Element, style: &ElementStyle, suggested_size: Size, inline_allocator: &mut InlineAllocator) -> Size {
    // for inline nodes
    // the returned height is the "added" height related to prev sibling

    let (margin, _border, _padding, content, _non_auto_width, non_auto_height) = box_sizing::get_sizes(element, style, suggested_size);

    let width = {
        let (_, max) = element.position_offset.min_max_width();
        if max > content.width() { content.width() } else { max }
    };

    let child_suggested_size = Size::new(content.width(), f64::NAN);
    let prev_filled_height = inline_allocator.get_current_height();
    let mut child_inline_allocator = InlineAllocator::new();
    child_inline_allocator.reset(element.node_mut(), &InlineAllocatorState::new(width, style.get_text_align()));

    let mut child_requested_height = 0.;
    if element.is_terminated() {
        element.content_mut().suggest_size(child_suggested_size, &mut child_inline_allocator, style);
        child_requested_height = child_inline_allocator.get_current_height();
    } else {
        let node = element.node_mut();
        node.for_each_child_mut(|child| {
            let size = child.position_offset.suggest_size(child_suggested_size, &mut child_inline_allocator, true, false);
            child_requested_height += size.height();
        });
    }

    let height = child_requested_height;
    let baseline_offset = height / 2.; // FIXME impl vertical-align
    let (left, inline_baseline_top) = {
        let node = element.node_mut();
        child_inline_allocator.end(node);
        inline_allocator.start_node(node, height, baseline_offset);
        inline_allocator.add_width(node, width, true)
    }.into();
    element.position_offset.inline_position_offset = Size::new(left, inline_baseline_top - baseline_offset - prev_filled_height);
    // TODO handle adjust_baseline_offset and adjust_text_align_offset

    let height = if !non_auto_height {
        margin.height() + inline_allocator.get_current_height() - prev_filled_height
    } else {
        margin.height()
    };
    Size::new(f64::NAN, height)
}

#[inline]
pub fn allocate_position(element: &mut Element, _style: &ElementStyle, allocated_point: Point, relative_point: Point) -> (Point, Bounds) {
    let requested_size = element.position_offset.requested_size();
    let mut drawing_bounds = Bounds::new(0., 0., requested_size.width(), requested_size.height());
    if element.content().is_terminated() {
        drawing_bounds.union(&element.content().drawing_bounds());
    } else {
        let mut current_top = 0.;
        let node = element.node_mut();
        node.for_each_child_mut(|child| {
            let requested_size = child.position_offset.requested_size();
            let child_bounds = child.position_offset.allocate_position(
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
