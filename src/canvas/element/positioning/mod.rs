use super::style::{DisplayType, PositionType, DEFAULT_F64};
use super::{Element};
use rc_forest::ForestNode;

mod position_types;
pub use self::position_types::{Position, Point, Size, Bounds};
mod inline_allocator;
pub(crate) use self::inline_allocator::InlineAllocator;
mod box_sizing;
mod inline;
mod block;
mod absolute;
mod flex;
mod none;

// position offset

#[derive(Debug)]
pub struct PositionOffset {
    element: *mut Element,
    suggested_size: Size,
    relative_size: Size,
    requested_size: Size,
    allocated_point: Point, // left-top corner relative to content box of parent node
    relative_point: Point, // left-top corner relative to content box of relative node
    drawing_bounds: Bounds, // drawing bounds relative to content box of parent node
}

impl PositionOffset {
    pub fn new() -> Self {
        PositionOffset {
            element: 0 as *mut Element,
            suggested_size: Size::new(0., 0.),
            relative_size: Size::new(0., 0.),
            requested_size: Size::new(0., 0.),
            allocated_point: Point::new(0., 0.),
            relative_point: Point::new(0., 0.),
            drawing_bounds: Bounds::new(0., 0., 0., 0.),
        }
    }
    #[inline]
    pub fn associate_element(&mut self, element: *mut Element) {
        self.element = element;
    }
    #[inline]
    fn element_mut<'a>(&'a mut self) -> &'a mut Element {
        unsafe { &mut *self.element }
    }
    #[inline]
    unsafe fn element_mut_unsafe<'a, 'b>(&'b mut self) -> &'a mut Element {
        &mut *self.element
    }
    #[inline]
    fn node_mut<'a>(&'a mut self) -> &'a mut ForestNode<Element> {
        self.element_mut().node_mut()
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

    pub(crate) fn suggest_size(&mut self, is_layout_dirty: bool, suggested_size: Size, inline_allocator: &mut InlineAllocator, handle_absolute: bool) -> Size {
        let element = unsafe { self.element_mut_unsafe() };
        let style = unsafe { element.style().clone_ref_unsafe() };

        let display = style.get_display();
        let position = style.get_position();
        let is_inline =
            !box_sizing::is_independent_positioning(style) &&
            (display == DisplayType::Inline || display == DisplayType::InlineBlock);

        // layout edge-cutting
        if !is_layout_dirty && !is_inline && suggested_size == self.suggested_size {
            return self.requested_size;
        }
        self.suggested_size = suggested_size;

        let requested_size = {
            if display == DisplayType::None {
                none::suggest_size(element, style, suggested_size, inline_allocator, handle_absolute)
            } else if is_inline {
                inline::suggest_size(element, style, suggested_size, inline_allocator, handle_absolute)
            } else if box_sizing::is_independent_positioning(style) {
                suggested_size
            } else {
                match display {
                    DisplayType::Flex => {
                        flex::suggest_size(element, style, suggested_size, inline_allocator, handle_absolute)
                    },
                    _ => {
                        block::suggest_size(element, style, suggested_size, inline_allocator, handle_absolute)
                    },
                }
            }
        };

        if handle_absolute && position != PositionType::Static {
            if is_layout_dirty || requested_size != self.relative_size {
                let mut ia = InlineAllocator::new();
                let node = element.node_mut();
                for child in node.clone_children().iter() {
                    let child = child.deref_mut_with(node);
                    child.suggest_size_absolute(requested_size, &mut ia);
                }
            }
        }

        self.requested_size = requested_size;
        debug!("Suggested size for {:?} with {:?}, requested {:?}", element, self.suggested_size, self.requested_size);
        requested_size
    }

    pub(crate) fn suggest_size_absolute(&mut self, is_layout_dirty: bool, relative_size: Size, inline_allocator: &mut InlineAllocator) {
        let element = unsafe { self.element_mut_unsafe() };
        let style = unsafe { element.style().clone_ref_unsafe() };

        // layout edge-cutting
        if !is_layout_dirty && relative_size == self.relative_size {
            return;
        }
        self.relative_size = relative_size;

        if box_sizing::is_independent_positioning(style) {
            absolute::suggest_size(element, style, relative_size, inline_allocator);
        } else {
            let node = element.node_mut();
            for child in node.clone_children().iter() {
                let child = child.deref_mut_with(node);
                child.suggest_size_absolute(relative_size, inline_allocator);
            }
        }
    }

    pub(crate) fn allocate_position(&mut self, is_layout_dirty: bool, allocated_point: Point, relative_point: Point) -> Bounds {
        let element = unsafe { self.element_mut_unsafe() };
        let style = unsafe { element.style().clone_ref_unsafe() };

        let display = style.get_display();
        let position = style.get_position();
        let is_inline =
            !box_sizing::is_independent_positioning(style) &&
            (display == DisplayType::Inline || display == DisplayType::InlineBlock);

        // layout edge-cutting
        if !is_layout_dirty && !is_inline && allocated_point == self.allocated_point && relative_point == self.relative_point {
            return self.drawing_bounds
        }
        self.allocated_point = allocated_point;

        let relative_point = match position {
            PositionType::Static => relative_point,
            _ => allocated_point, // TODO late handling independent_positioning
        };
        self.relative_point = relative_point;

        let drawing_bounds = {
            if display == DisplayType::None {
                none::allocate_position(element, style, allocated_point, relative_point)
            } else if is_inline {
                inline::allocate_position(element, style, allocated_point, relative_point)
            } else if box_sizing::is_independent_positioning(style) {
                absolute::allocate_position(element, style, allocated_point, relative_point)
            } else {
                match display {
                    DisplayType::Flex => {
                        flex::allocate_position(element, style, allocated_point, relative_point)
                    },
                    _ => {
                        block::allocate_position(element, style, allocated_point, relative_point)
                    },
                }
            }
        };

        self.drawing_bounds = drawing_bounds;
        debug!("Allocated position for {:?} with {:?} drawing bounds {:?}", element, self.allocated_point, self.drawing_bounds);
        drawing_bounds
    }
}
