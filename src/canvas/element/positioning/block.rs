use super::super::{Element, ElementStyle, DEFAULT_F64};
use super::{Point, Size, Position, Bounds, InlineAllocator, InlineAllocatorState, box_sizing};

#[inline]
pub fn get_min_max_width(element: &mut Element, style: &ElementStyle, inline_allocator: &mut InlineAllocator) -> (f64, f64) {
    let (_margin, _border, _padding, content) = box_sizing::get_sizes(style, Size::new(suggested_size.width(), DEFAULT_F64));

}

#[inline]
pub fn suggest_size(element: &mut Element, style: &ElementStyle, suggested_size: Size, inline_allocator: &mut InlineAllocator) -> Size {
    let (margin, _border, _padding, content) = box_sizing::get_sizes(style, Size::new(suggested_size.width(), DEFAULT_F64));
    let child_suggested_size = content;

    let state = inline_allocator.state().clone();
    inline_allocator.reset(element.node_mut(), &InlineAllocatorState::new(content.width(), style.get_text_align()));
    let mut child_requested_height = 0.;
    if element.is_terminated() {
        let _size = element.content_mut().suggest_size(child_suggested_size, inline_allocator, style);
        child_requested_height = inline_allocator.get_current_height();
    } else {
        let node = element.node_mut();
        node.for_each_child_mut(|child| {
            let size = child.suggest_size(child_suggested_size, inline_allocator, true);
            child_requested_height += size.height();
        });
    }
    inline_allocator.reset(element.node_mut(), &state);

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
