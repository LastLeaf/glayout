use super::style::{DisplayType, PositionType, DEFAULT_F64};
use super::{Element};
use rc_forest::ForestNode;

mod position_types;
pub use self::position_types::{Position, Point, Size, Bounds};
mod inline_allocator;
pub use self::inline_allocator::{InlineSize, InlineAllocator};

// position offset

#[derive(Debug)]
pub struct PositionOffset {
    element: *mut Element,
    suggested_size: Size,
    requested_size: Size,
    allocated_position: Position,
    drawing_bounds: Bounds,
}

impl PositionOffset {
    pub fn new() -> Self {
        PositionOffset {
            element: 0 as *mut Element,
            suggested_size: Size::new(0., 0.),
            requested_size: Size::new(0., 0.),
            allocated_position: Position::new(0., 0., 0., 0.),
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
    pub fn requested_size(&self) -> Size {
        self.requested_size
    }
    #[inline]
    pub fn allocated_position(&self) -> Position {
        self.allocated_position
    }
    #[inline]
    pub fn drawing_bounds(&self) -> Bounds {
        self.drawing_bounds
    }
    #[inline]
    fn merge_drawing_bounds(&mut self, child_bounds: Bounds, offset: Size) {
        self.drawing_bounds.union(&(child_bounds + offset));
    }

    pub fn suggest_size(&mut self, is_dirty: bool, suggested_size: Size, inline_allocator: &mut InlineAllocator) -> Size {
        let element = unsafe { self.element_mut_unsafe() };
        let style = unsafe { element.style().clone_ref_unsafe() };

        if !is_dirty && suggested_size == self.suggested_size {
            if (style.get_position() == PositionType::Static || style.get_position() == PositionType::Relative) &&
                (style.get_display() == DisplayType::Inline || style.get_display() == DisplayType::InlineBlock) {
                // inline nodes cannot be edge cutted
            } else {
                return self.requested_size
            }
        }
        self.suggested_size = suggested_size;
        let request_width;
        let mut request_height = 0.;
        let suggested_width = if style.get_width() == DEFAULT_F64 { suggested_size.width() } else { style.get_width() };
        let suggested_height = if style.get_height() == DEFAULT_F64 { suggested_size.height() } else { style.get_height() };

        // suggest size for children
        match style.get_position() {
            PositionType::Static | PositionType::Relative | PositionType::Sticky => {
                match style.get_display() {

                    DisplayType::None => {
                        request_width = 0.;
                        request_height = 0.;
                    },
                    DisplayType::Block | DisplayType::Flex => {
                        request_width = suggested_width;
                        let child_suggested_width = suggested_width - style.get_margin_left() - style.get_margin_right() - style.get_padding_left() - style.get_padding_right();
                        let child_suggested_size = Size::new(child_suggested_width, 0.);
                        request_height += style.get_margin_top() + style.get_padding_top();
                        inline_allocator.reset(child_suggested_width, style.get_text_align());
                        if element.is_terminated() {
                            let size = element.content_mut().suggest_size(child_suggested_size, inline_allocator, style);
                            request_height += size.height();
                        } else {
                            for child in element.node().iter() {
                                let size = child.deref_mut_with(self.node_mut()).suggest_size(child_suggested_size, inline_allocator);
                                request_height += size.height();
                            }
                        }
                        inline_allocator.reset(child_suggested_width, style.get_text_align());
                        request_height += style.get_margin_bottom() + style.get_padding_bottom();
                        if style.get_height() != DEFAULT_F64 {
                            request_height = suggested_height;
                        }
                    },
                    DisplayType::Inline | DisplayType::InlineBlock => {
                        request_width = suggested_width;
                        if element.content().is_terminated() {
                            let size = element.content_mut().suggest_size(suggested_size, inline_allocator, style);
                            request_height += size.height();
                        } else {
                            for child in element.node().iter() {
                                let size = child.deref_mut_with(self.node_mut()).suggest_size(suggested_size, inline_allocator);
                                request_height += size.height();
                            }
                        }
                        if style.get_height() != DEFAULT_F64 {
                            request_height = suggested_height;
                        }
                    },
                };
            },
            PositionType::Absolute | PositionType::Fixed => {
                match style.get_display() {
                    DisplayType::None => {
                        request_width = 0.;
                        request_height = 0.;
                    },
                    _ => {
                        request_width = if style.get_width() == DEFAULT_F64 { suggested_size.width() } else { style.get_width() };
                        request_height = if style.get_height() == DEFAULT_F64 { suggested_size.height() } else { style.get_height() };
                        let absolute_request_width = suggested_width; // FIXME calc it!
                        inline_allocator.reset(absolute_request_width, style.get_text_align());
                        if element.content().is_terminated() {
                            element.content_mut().suggest_size(Size::new(suggested_size.width(), 0.), inline_allocator, style);
                        } else {
                            let node = element.node_mut();
                            for child in node.clone_children().iter() {
                                child.deref_mut_with(node).suggest_size(Size::new(suggested_size.width(), 0.), inline_allocator);
                            }
                        }
                        inline_allocator.reset(absolute_request_width, style.get_text_align());
                    }
                };
            },
        }

        self.requested_size = Size::new(request_width, request_height);
        // debug!("Suggested size for {} with ({}, {}), requested ({}, {})", element, suggested_size.width(), suggested_size.height(), self.requested_size.width(), self.requested_size.height());
        self.requested_size
    }
    pub fn allocate_position(&mut self, is_dirty: bool, allocated_position: Position) -> Bounds {
        if !is_dirty && allocated_position == self.allocated_position {
            return self.drawing_bounds
        }
        let element = unsafe { self.element_mut_unsafe() };
        let style = unsafe { element.style().clone_ref_unsafe() };

        self.allocated_position = allocated_position;
        self.drawing_bounds = allocated_position.into(); // FIXME drawing bounds should ignore margin
        if element.content().is_terminated() {
            let child_bounds = element.content().drawing_bounds();
            self.merge_drawing_bounds(child_bounds, Size::new(0., 0.));
        } else {
            let mut current_height = style.get_margin_top() + style.get_padding_top();
            let current_left = style.get_margin_left() + style.get_padding_left();
            let mut current_inline_height = 0.;
            let current_width = allocated_position.width() - current_left - style.get_margin_right() - style.get_padding_right();
            let node = element.node_mut();
            for child_rc in node.clone_children().iter() {
                let child = child_rc.deref_mut_with(node);
                let requested_size = child.requested_size();
                let child_style = unsafe { child.style().clone_ref_unsafe() };

                match child_style.get_position() {
                    PositionType::Static | PositionType::Relative | PositionType::Sticky => {
                        match child_style.get_display() {
                            DisplayType::None => {
                                /* do nothing */
                            },
                            DisplayType::Block | DisplayType::Flex => {
                                if current_inline_height > 0. {
                                    current_height += current_inline_height;
                                    current_inline_height = 0.;
                                }
                                let child_bounds = child.allocate_position(Position::new(current_left, current_height, current_width, requested_size.height()));
                                self.merge_drawing_bounds(child_bounds, Size::new(current_left, current_height));
                                current_height += requested_size.height();
                            },
                            DisplayType::Inline | DisplayType::InlineBlock => {
                                // the allocated height for inline nodes should be zero, so that drawing_bounds is empty for inline nodes themselves
                                let child_bounds = child.allocate_position(Position::new(current_left, current_height, current_width, 0.));
                                self.merge_drawing_bounds(child_bounds, Size::new(current_left, current_height));
                                current_inline_height += requested_size.height();
                            },
                        };
                    },
                    PositionType::Absolute | PositionType::Fixed => {
                        match child_style.get_display() {
                            DisplayType::None => {
                                /* do nothing */
                            },
                            _ => {
                                let left = style.get_margin_left() + if child_style.get_left() == DEFAULT_F64 { 0. } else { child_style.get_left() };
                                let top = style.get_margin_top() + if child_style.get_top() == DEFAULT_F64 { 0. } else { child_style.get_top() };
                                let child_bounds = child.allocate_position(Position::new(left, top, requested_size.width(), requested_size.height()));
                                self.merge_drawing_bounds(child_bounds, Size::new(left, top));
                            }
                        };
                    },
                }

            }
        }
        // debug!("Allocated position for {} with {:?} drawing bounds {:?}", element, allocated_position, self.drawing_bounds);
        self.drawing_bounds
    }
}
