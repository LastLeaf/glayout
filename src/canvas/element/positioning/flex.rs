use std::f64;
use super::super::{Element, ElementStyle};
use super::{Point, Size, Position, Bounds, InlineAllocator, box_sizing};

#[inline]
pub fn get_min_max_width(element: &mut Element, style: &ElementStyle, inline_allocator: &mut InlineAllocator) -> (f64, f64) {
    let (offset, non_auto_width) = box_sizing::get_h_offset(style);

    inline_allocator.reset_with_current_state(element.node_mut());
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
            max_width += max;
        });
        (min_width + offset, max_width + offset)
    };
    inline_allocator.reset_with_current_state(element.node_mut());

    min_max_width
}

#[derive(Debug)]
struct FlexParams {
    is_independent_positioning: bool,
    width: f64,
    min: f64,
    max: f64,
    basis: f64,
    flex_grow: f32,
    flex_shrink: f32,
}

#[inline]
pub fn suggest_size(element: &mut Element, style: &ElementStyle, suggested_size: Size, inline_allocator: &mut InlineAllocator) -> Size {
    let (margin, _border, padding, content, _non_auto_width, non_auto_height) = box_sizing::get_sizes(element, style, suggested_size);

    // get min, max, flex-basis from each child
    let node = element.node_mut();
    let mut child_min_max_basis_flex = Vec::with_capacity(node.len());
    let mut min_total = 0.;
    let mut basis_total = 0.;
    let mut flex_grow_total = 0.;
    let mut flex_shrink_total = 0.;
    node.for_each_child_mut(|child| {
        if box_sizing::is_independent_positioning(style) {
            child_min_max_basis_flex.push(FlexParams {
                is_independent_positioning: true,
                width: 0.,
                min: 0.,
                max: 0.,
                basis: 0.,
                flex_grow: 0.,
                flex_shrink: 0.,
            });
        } else {
            let (min, max) = child.position_offset.min_max_width();
            let basis = if child.style().get_flex_basis().is_finite() {
                child.style().get_flex_basis()
            } else {
                max
            };
            let max = f64::MAX; // FIXME should be CSS max-width
            let flex_grow = child.style().get_flex_grow(); // FIXME should be CSS flex-grow
            let flex_shrink = child.style().get_flex_shrink(); // FIXME should be CSS flex-shrink
            flex_grow_total += flex_grow;
            flex_shrink_total += flex_shrink;
            child_min_max_basis_flex.push(FlexParams {
                is_independent_positioning: false,
                width: basis,
                min,
                max,
                basis,
                flex_grow,
                flex_shrink,
            });
            min_total += min;
            basis_total += basis;
        }
    });

    // calculate most suitable width for each child
    if basis_total < content.width() {
        // need growing
        let mut space = content.width() - basis_total;
        let mut next_space = space;
        while space > 1e-6 && flex_grow_total >= 1e-6 {
            for params in child_min_max_basis_flex.iter_mut() {
                if params.is_independent_positioning {
                    continue;
                }
                let ratio = (params.flex_grow / flex_grow_total) as f64;
                let s = if params.max < space * ratio {
                    flex_grow_total -= params.flex_grow;
                    params.flex_grow = 0.;
                    params.max
                } else {
                    space * ratio
                };
                params.width += s;
                next_space -= s;
            }
            space = next_space;
        }
    } else {
        // need shrinking
        let mut space = basis_total - content.width();
        let mut next_space = space;
        while space > 1e-6 && flex_shrink_total >= 1e-6 {
            for params in child_min_max_basis_flex.iter_mut() {
                if params.is_independent_positioning {
                    continue;
                }
                let ratio = (params.flex_shrink / flex_shrink_total) as f64;
                let s = if params.min > space * ratio {
                    flex_shrink_total -= params.flex_shrink;
                    params.flex_shrink = 0.;
                    params.min
                } else {
                    space * ratio
                };
                params.width -= s;
                next_space -= s;
            }
            space = next_space;
        }
    }

    // suggest size to children
    let mut child_requested_height = 0.;
    let range = 0..node.len();
    for i in range {
        let child = node.child_mut(i).unwrap();
        if child_min_max_basis_flex[i].is_independent_positioning {
            continue;
        }
        let size = child.position_offset.suggest_size(Size::new(child_min_max_basis_flex[i].width, content.height()), inline_allocator, false, false);
        if child_requested_height < size.height() { child_requested_height = size.height() };
    }
    node.position_offset.content_size = Size::new(0., child_requested_height);

    element.position_offset.background_rect = Position::new(
        0.,
        0.,
        padding.width(),
        if non_auto_height { padding.height() } else { padding.height() + child_requested_height },
    );
    if !non_auto_height {
        margin + Size::new(0., child_requested_height)
    } else {
        margin
    }
}

#[inline]
pub fn allocate_position(element: &mut Element, style: &ElementStyle, allocated_point: Point, relative_point: Point) -> (Point, Bounds) {
    let requested_size = element.position_offset.requested_size;
    let suggested_size = element.position_offset.suggested_size;
    let (_margin, _border, padding, content) = box_sizing::get_offsets(style, suggested_size, requested_size);
    element.position_offset.background_rect = Position::new(
        padding.left(),
        padding.top(),
        element.position_offset.background_rect.width(),
        element.position_offset.background_rect.height(),
    );
    let allocated_position = Position::from((allocated_point, requested_size));
    let mut drawing_bounds: Bounds = allocated_position.into();
    if element.content().is_terminated() {
        drawing_bounds.union(&element.content().drawing_bounds());
    } else {
        let mut current_left = content.left();
        let current_height = element.position_offset.content_size.height();
        let node = element.node_mut();
        node.for_each_child_mut(|child| {
            let align_self_offset = (current_height - child.position_offset.requested_size.height()) / 2.;
            let current_top = content.top() + align_self_offset;
            let child_bounds = child.position_offset.allocate_position(
                Point::new(current_left, current_top),
                relative_point + Size::new(-current_left, -current_top)
            );
            drawing_bounds.union(&child_bounds);
            if !box_sizing::is_independent_positioning(child.style()) {
                current_left += child.position_offset.suggested_size.width();
            }
        });
    }
    (allocated_point, drawing_bounds)
}
