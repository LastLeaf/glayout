use std::f64;
use super::style::*;
use super::{Element};
use rc_forest::ForestNode;

mod position_types;
pub use self::position_types::{Position, Point, Size, Bounds};
mod inline_allocator;
pub(crate) use self::inline_allocator::{InlineAllocator, InlineAllocatorState};
mod box_sizing;
mod inline;
mod inline_block;
mod block;
mod flex;
mod none;

// position offset

#[derive(Debug)]
pub struct PositionOffset {
    element: *mut Element,
    suggested_size: Size,
    relative_size: Size,
    requested_size: Size,
    inline_position_offset: Size, // left-top corner of the inline/inline_block relative to base position
    content_size: Size, // the content size for flex
    allocated_point: Point, // left-top corner relative to content box of parent node
    relative_point: Point, // left-top corner relative to content box of relative node
    drawing_bounds: Bounds, // drawing bounds relative to content box of parent node
    min_max_width: (f64, f64), // min and max width
    position_dirty: bool,
    min_max_width_dirty: bool,
    background_rect: Position,
}

impl PositionOffset {
    pub fn new() -> Self {
        PositionOffset {
            element: 0 as *mut Element,
            suggested_size: Size::new(0., 0.),
            relative_size: Size::new(0., 0.),
            requested_size: Size::new(0., 0.),
            inline_position_offset: Size::new(0., 0.),
            content_size: Size::new(0., 0.),
            allocated_point: Point::new(0., 0.),
            relative_point: Point::new(0., 0.),
            drawing_bounds: Bounds::new(0., 0., 0., 0.),
            min_max_width: (0., 0.),
            position_dirty: true,
            min_max_width_dirty: true,
            background_rect: Position::new(0., 0., 0., 0.),
        }
    }
    #[inline]
    pub fn associate_element(&mut self, element: *mut Element) {
        self.element = element;
    }
    #[inline]
    fn _element_mut<'a>(&'a mut self) -> &'a mut Element {
        unsafe { &mut *self.element }
    }
    #[inline]
    unsafe fn element_mut_unsafe<'a, 'b>(&'b mut self) -> &'a mut Element {
        &mut *self.element
    }
    #[inline]
    fn _node_mut<'a>(&'a mut self) -> &'a mut ForestNode<Element> {
        self._element_mut().node_mut()
    }

    #[inline]
    pub(crate) fn get_and_mark_dirty(&mut self) -> bool {
        let ret = self.position_dirty;
        self.position_dirty = true;
        self.min_max_width_dirty = true;
        ret
    }
    #[inline]
    pub(crate) fn is_dirty(&self) -> bool {
        self.position_dirty
    }

    #[inline]
    pub(crate) fn requested_size(&self) -> Size {
        self.requested_size
    }
    #[inline]
    pub(crate) fn allocated_point(&self) -> Point {
        self.allocated_point
    }
    #[inline]
    pub(crate) fn drawing_bounds(&self) -> Bounds {
        self.drawing_bounds
    }
    #[inline]
    fn merge_drawing_bounds(&mut self, child_bounds: Bounds, offset: Size) {
        self.drawing_bounds.union(&(child_bounds + offset));
    }

    fn min_max_width_dfs(&mut self, inline_allocator: &mut InlineAllocator, handle_independent_positioning: bool) -> (f64, f64) {
        let element = unsafe { self.element_mut_unsafe() };
        let style = unsafe { element.style().clone_ref_unsafe() };

        if !handle_independent_positioning && box_sizing::is_independent_positioning(style) {
            return (0., 0.);
        }

        let display = style.get_display();
        let is_inline =
            !box_sizing::is_independent_positioning(style) &&
            (display == DisplayType::Inline || display == DisplayType::InlineBlock);

        // layout edge-cutting
        if !is_inline {
            inline_allocator.reset_with_current_state(element.node_mut());
            if !self.min_max_width_dirty {
                return self.min_max_width;
            }
        }
        element.set_base_size_and_font_size(Size::new(0., 0.), element.style.get_font_size());

        self.min_max_width = {
            if display == DisplayType::None {
                none::get_min_max_width(element, style, inline_allocator)
            } else if is_inline {
                if display == DisplayType::InlineBlock {
                    inline_block::get_min_max_width(element, style, inline_allocator)
                } else {
                    inline::get_min_max_width(element, style, inline_allocator)
                }
            } else if display == DisplayType::Flex && !element.is_terminated() {
                flex::get_min_max_width(element, style, inline_allocator)
            } else {
                block::get_min_max_width(element, style, inline_allocator)
            }
        };

        self.min_max_width_dirty = false;
        debug!("Get min max width from {:?}, get {:?}", element, self.min_max_width);
        self.min_max_width
    }

    fn min_max_width(&mut self) -> (f64, f64) {
        self.min_max_width_dfs(&mut InlineAllocator::new(), true)
    }

    pub(crate) fn suggest_size(&mut self, suggested_size: Size, inline_allocator: &mut InlineAllocator, enable_inline: bool, handle_independent_positioning: bool) -> Size {
        let element = unsafe { self.element_mut_unsafe() };
        let style = unsafe { element.style().clone_ref_unsafe() };

        let is_independent_positioning = box_sizing::is_independent_positioning(style);
        if !handle_independent_positioning && is_independent_positioning {
            return Size::new(0., 0.);
        }

        let display = style.get_display();
        let position = style.get_position();
        let is_inline =
            enable_inline &&
            !box_sizing::is_independent_positioning(style) &&
            (display == DisplayType::Inline || display == DisplayType::InlineBlock);

        // layout edge-cutting
        if !is_inline {
            inline_allocator.reset_with_current_state(element.node_mut());
            if !self.position_dirty && suggested_size == self.suggested_size {
                return self.requested_size;
            }
        }
        self.suggested_size = suggested_size;
        element.set_base_size_and_font_size(suggested_size, element.style.get_font_size());

        // collapse height if not specified
        let flex_direction = style.get_flex_direction();
        let is_vertical_flex = flex_direction == FlexDirectionType::Column || flex_direction == FlexDirectionType::ColumnReverse;
        let keep_height =
            is_independent_positioning ||
            (display == DisplayType::Flex && !element.is_terminated() && is_vertical_flex);
        let suggested_size = if keep_height {
            suggested_size
        } else {
            Size::new(suggested_size.width(), f64::NAN)
        };

        let requested_size = {
            if display == DisplayType::None {
                none::suggest_size(element, style, suggested_size, inline_allocator)
            } else if is_inline {
                if display == DisplayType::InlineBlock {
                    inline_block::suggest_size(element, style, suggested_size, inline_allocator)
                } else {
                    inline::suggest_size(element, style, suggested_size, inline_allocator)
                }
            } else if display == DisplayType::Flex && !element.is_terminated() {
                if is_vertical_flex {
                    // TODO impl vertical flex
                    unimplemented!("vertical flex is not supported yet")
                } else {
                    flex::suggest_size(element, style, suggested_size, inline_allocator)
                }
            } else {
                block::suggest_size(element, style, suggested_size, inline_allocator)
            }
        };

        if position != PositionType::Static {
            if self.position_dirty || requested_size != self.relative_size {
                let mut ia = InlineAllocator::new();
                let node = element.node_mut();
                node.for_each_child_mut(|child| {
                    child.position_offset.suggest_size_absolute(requested_size, &mut ia);
                });
            }
        }

        self.requested_size = requested_size;
        debug!("Suggested size for {:?} with {:?}, requested {:?}", element, self.suggested_size, self.requested_size);
        requested_size
    }

    pub(crate) fn suggest_size_absolute(&mut self, relative_size: Size, inline_allocator: &mut InlineAllocator) {
        let element = unsafe { self.element_mut_unsafe() };
        let style = unsafe { element.style().clone_ref_unsafe() };

        // layout edge-cutting
        if !self.position_dirty && relative_size == self.relative_size {
            return;
        }
        self.relative_size = relative_size;

        if box_sizing::is_independent_positioning(style) {
            element.position_offset.suggest_size(relative_size, inline_allocator, false, true);
        } else {
            let node = element.node_mut();
            node.for_each_child_mut(|child| {
                child.position_offset.suggest_size_absolute(relative_size, inline_allocator);
            })
        }
    }

    pub(crate) fn allocate_position(&mut self, allocated_point: Point, relative_point: Point) -> Bounds {
        let element = unsafe { self.element_mut_unsafe() };
        let style = unsafe { element.style().clone_ref_unsafe() };

        let allocated_point = if box_sizing::is_independent_positioning(style) {
            self.allocate_position_absolute(style, relative_point)
        } else {
            allocated_point
        };

        let display = style.get_display();
        let position = style.get_position();
        let is_inline =
            !box_sizing::is_independent_positioning(style) &&
            (display == DisplayType::Inline || display == DisplayType::InlineBlock);

        // layout edge-cutting
        if !self.position_dirty && !is_inline && allocated_point == self.allocated_point && relative_point == self.relative_point {
            return self.drawing_bounds
        }

        let relative_point = match position {
            PositionType::Static => relative_point,
            _ => allocated_point, // TODO late handling independent_positioning
        };
        self.relative_point = relative_point;

        let (allocated_point, drawing_bounds) = {
            if display == DisplayType::None {
                none::allocate_position(element, style, allocated_point, relative_point)
            } else if is_inline {
                if display == DisplayType::InlineBlock {
                    inline_block::allocate_position(element, style, allocated_point, relative_point)
                } else {
                    inline::allocate_position(element, style, allocated_point, relative_point)
                }
            } else if display == DisplayType::Flex && !element.is_terminated() {
                flex::allocate_position(element, style, allocated_point, relative_point)
            } else {
                block::allocate_position(element, style, allocated_point, relative_point)
            }
        };

        self.allocated_point = allocated_point;
        self.drawing_bounds = drawing_bounds;
        self.position_dirty = false;
        debug!("Allocated position for {:?} with {:?} drawing bounds {:?}", element, self.allocated_point, self.drawing_bounds);
        drawing_bounds
    }

    fn allocate_position_absolute(&mut self, style: &super::ElementStyle, relative_point: Point) -> Point {
        let left = if style.get_left().is_finite() {
            style.get_left()
        } else {
            0.
        };
        let top = if style.get_top().is_finite() {
            style.get_top()
        } else {
            0.
        };
        Point::new(
            relative_point.left() + left,
            relative_point.top() + top,
        )
    }

    pub(super) fn get_background_rect(&mut self) -> Position {
        self.background_rect
    }
}
