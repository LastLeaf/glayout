use std::f64;
use super::super::{Element, ElementStyle};
use super::{Point, Size, Bounds, InlineAllocator, box_sizing};

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

    let child_suggested_size = suggested_size;

    let mut child_requested_height = 0.;
    if element.is_terminated() {
        let prev_height = inline_allocator.get_current_height();
        element.content_mut().suggest_size(child_suggested_size, inline_allocator, style);
        child_requested_height = inline_allocator.get_current_height() - prev_height;
    } else {
        let node = element.node_mut();
        node.for_each_child_mut(|child| {
            let size = child.position_offset.suggest_size(child_suggested_size, inline_allocator, true, false);
            child_requested_height += size.height();
        });
    }

    Size::new(f64::NAN, child_requested_height)
}

#[inline]
pub fn allocate_position(element: &mut Element, _style: &ElementStyle, allocated_point: Point, relative_point: Point) -> (Point, Bounds) {
    let requested_size = element.position_offset.requested_size();
    let drawing_bounds = if element.content().is_terminated() {
        element.content().drawing_bounds()
    } else {
        let mut drawing_bounds = Bounds::new(requested_size.width(), allocated_point.top(), requested_size.width(), allocated_point.top());
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
        drawing_bounds
    };
    (allocated_point, drawing_bounds)
}
