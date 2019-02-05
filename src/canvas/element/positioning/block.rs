use super::super::{Element, ElementStyle, DEFAULT_F64};
use super::{Point, Size, Position, Bounds, InlineAllocator, box_sizing};

#[inline]
pub fn suggest_size(element: &mut Element, style: &ElementStyle, suggested_size: Size, inline_allocator: &mut InlineAllocator, handle_absolute: bool) -> Size {
    // NOTE the returned size is the "added" size related to prev sibling
    let (margin, _border, _padding, content) = box_sizing::get_sizes(style, Size::new(suggested_size.width(), 0.));
    let child_suggested_size = content;

    let mut child_requested_height = 0.;
    if element.is_terminated() {
        inline_allocator.reset(element.node_mut(), content.width(), style.get_text_align());
        let _size = element.content_mut().suggest_size(child_suggested_size, inline_allocator, style);
        child_requested_height = inline_allocator.get_current_height();
    } else {
        let node = element.node_mut();
        for child in node.clone_children().iter() {
            let size = child.deref_mut_with(node).suggest_size(child_suggested_size, inline_allocator, handle_absolute);
            child_requested_height += size.height();
        }
    }

    if style.get_height() == DEFAULT_F64 {
        margin + Size::new(0., child_requested_height)
    } else {
        margin
    }
}

#[inline]
pub fn allocate_position(element: &mut Element, style: &ElementStyle, allocated_point: Point, relative_point: Point) -> Bounds {
    let (_margin, _border, _padding, content) = box_sizing::get_offsets(style);
    let requested_size = element.requested_size();
    let allocated_position = Position::from((allocated_point, requested_size));
    let mut drawing_bounds: Bounds = allocated_position.into();
    if element.content().is_terminated() {
        drawing_bounds.union(&element.content().drawing_bounds());
    } else {
        let mut current_top = content.top();
        let node = element.node_mut();
        for child in node.clone_children().iter() {
            let child = child.deref_mut_with(node);
            let requested_size = child.requested_size();
            let child_bounds = child.allocate_position(
                Point::new(0., current_top),
                relative_point + Size::new(0., -current_top)
            ) + content.into();
            drawing_bounds.union(&child_bounds);
            if !box_sizing::is_independent_positioning(child.style()) {
                current_top += requested_size.height();
            }
        }
    };
    drawing_bounds
}
