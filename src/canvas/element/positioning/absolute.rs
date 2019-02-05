use super::super::{Element, ElementStyle, DEFAULT_F64};
use super::{Point, Size, Position, Bounds, InlineAllocator, box_sizing};

#[inline]
pub fn suggest_size(element: &mut Element, style: &ElementStyle, relative_size: Size, inline_allocator: &mut InlineAllocator) {
    // NOTE the returned size is the "added" size related to prev sibling
    let width = if style.get_width() != DEFAULT_F64 {
        style.get_width()
    } else {
        let mut width = relative_size.width();
        if style.get_left() != DEFAULT_F64 {
            width -= style.get_left();
        }
        if style.get_right() != DEFAULT_F64 {
            width -= style.get_right();
        }
        width
    };
    let height = if style.get_height() != DEFAULT_F64 {
        style.get_height()
    } else {
        let mut height = relative_size.height();
        if style.get_top() != DEFAULT_F64 {
            height -= style.get_top();
        }
        if style.get_bottom() != DEFAULT_F64 {
            height -= style.get_bottom();
        }
        height
    };
    let style_padding_width = style.get_padding_left() + style.get_padding_right();
    let style_padding_height = style.get_padding_top() + style.get_padding_bottom();
    let style_border_width = style.get_border_left_width() + style.get_border_right_width();
    let style_border_height = style.get_border_top_width() + style.get_border_bottom_width();
    let content = Size::new(
        width - style_border_width - style_padding_width,
        height - style_border_height - style_padding_height,
    );
    element.suggest_size(content, inline_allocator, true);
}

#[inline]
pub fn allocate_position(element: &mut Element, style: &ElementStyle, allocated_point: Point, relative_point: Point) -> Bounds {
    let left = if style.get_left() != DEFAULT_F64 {
        style.get_left()
    } else {
        0.
    };
    let top = if style.get_top() != DEFAULT_F64 {
        style.get_top()
    } else {
        0.
    };
    let content = Point::new(
        relative_point.left() + left,
        relative_point.top() + top,
    );

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
                Point::new(0., -current_top)
            ) + content.into();
            drawing_bounds.union(&child_bounds);
            if !box_sizing::is_independent_positioning(child.style()) {
                current_top += requested_size.height();
            }
        }
    };
    drawing_bounds
}
