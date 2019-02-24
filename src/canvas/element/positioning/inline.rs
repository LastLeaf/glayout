use super::super::{Element, ElementStyle, DEFAULT_F64};
use super::{Point, Size, Bounds, InlineAllocator, box_sizing};

#[inline]
pub fn suggest_size(element: &mut Element, style: &ElementStyle, suggested_size: Size, inline_allocator: &mut InlineAllocator) -> Size {
    // for inline nodes
    // the returned width is the current end width of the inline allocation
    // the returned height is the "added" height related to prev sibling

    let child_suggested_size = Size::new(suggested_size.width(), 0.);

    let mut child_requested_height = 0.;
    if element.is_terminated() {
        let prev_height = inline_allocator.get_current_height();
        element.content_mut().suggest_size(child_suggested_size, inline_allocator, style);
        child_requested_height = inline_allocator.get_current_height() - prev_height;
    } else {
        let node = element.node_mut();
        node.for_each_child_mut(|child| {
            let size = child.suggest_size(child_suggested_size, inline_allocator, true);
            child_requested_height += size.height();
        });
    }

    Size::new(inline_allocator.get_current_line_width(), child_requested_height)
}

#[inline]
pub fn allocate_position(element: &mut Element, _style: &ElementStyle, allocated_point: Point, relative_point: Point) -> (Point, Bounds) {
    let requested_size = element.requested_size();
    let drawing_bounds = if element.content().is_terminated() {
        element.content().drawing_bounds()
    } else {
        let mut drawing_bounds = Bounds::new(requested_size.width(), allocated_point.top(), requested_size.width(), allocated_point.top());
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
        drawing_bounds
    };
    (allocated_point, drawing_bounds)
}
