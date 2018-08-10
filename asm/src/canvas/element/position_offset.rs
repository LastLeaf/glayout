use super::super::super::tree::TreeNodeRc;
use super::style::{DisplayType, PositionType, DEFAULT_F64};
use super::Element;

// position offset

#[derive(Default, Debug)]
pub struct PositionOffset {
    suggested_size: (f64, f64),
    requested_size: (f64, f64),
    allocated_position: (f64, f64, f64, f64),
    drawing_bounds: (f64, f64, f64, f64),
}

impl PositionOffset {
    pub fn new() -> Self {
        PositionOffset {
            ..Default::default()
        }
    }

    #[inline]
    pub fn requested_size(&self) -> (f64, f64) {
        self.requested_size
    }
    #[inline]
    pub fn allocated_position(&self) -> (f64, f64, f64, f64) {
        self.allocated_position
    }
    #[inline]
    pub fn drawing_bounds(&self) -> (f64, f64, f64, f64) {
        self.drawing_bounds
    }
    #[inline]
    fn merge_drawing_bounds(&mut self, child_bounds: &(f64, f64, f64, f64)) {
        if self.drawing_bounds.0 > child_bounds.0 { self.drawing_bounds.0 = child_bounds.0 }
        if self.drawing_bounds.1 > child_bounds.1 { self.drawing_bounds.1 = child_bounds.1 }
        if self.drawing_bounds.2 < child_bounds.2 { self.drawing_bounds.2 = child_bounds.2 }
        if self.drawing_bounds.3 < child_bounds.3 { self.drawing_bounds.3 = child_bounds.3 }
    }

    pub fn suggest_size(&mut self, is_dirty: bool, suggested_size: (f64, f64), inline_position_status: &mut InlinePositionStatus, element: &Element) -> (f64, f64) {
        if !is_dirty && suggested_size == self.suggested_size {
            // TODO is an inline node is dirty, then all inline nodes beside it are dirty
            return self.requested_size
        }
        self.suggested_size = suggested_size;
        let request_width;
        let mut request_height = 0.; // for inline nodes, request_height is the added height while appending the node
        let style = element.style();
        let suggested_width = if style.get_width() == DEFAULT_F64 { suggested_size.0 } else { style.get_width() };
        let suggested_height = if style.get_height() == DEFAULT_F64 { suggested_size.1 } else { style.get_height() };

        // suggest size for children
        match style.get_position() {
            PositionType::Static | PositionType::Relative => {
                match style.get_display() {
                    DisplayType::None => {
                        request_width = 0.;
                        request_height = 0.;
                    },
                    DisplayType::Block => {
                        request_width = suggested_width;
                        inline_position_status.reset(request_width);
                        if element.content().is_terminated() {
                            let (_, h) = element.content_mut().suggest_size((suggested_size.0, 0.), inline_position_status, &*element.style());
                            request_height += h;
                        } else {
                            for child in element.tree_node().iter_children() {
                                let (_, h) = child.elem().suggest_size((suggested_size.0, 0.), inline_position_status);
                                request_height += h;
                            }
                        }
                        inline_position_status.reset(request_width);
                        if style.get_height() != DEFAULT_F64 {
                            request_height = suggested_height;
                        }
                    },
                    DisplayType::Inline | DisplayType::InlineBlock => {
                        request_width = suggested_width;
                        if element.content().is_terminated() {
                            let (_, h) = element.content_mut().suggest_size((suggested_size.0, 0.), inline_position_status, &*element.style());
                            request_height += h;
                        } else {
                            for child in element.tree_node().iter_children() {
                                let (_, h) = child.elem().suggest_size((suggested_size.0, 0.), inline_position_status);
                                request_height += h;
                            }
                        }
                        if style.get_height() != DEFAULT_F64 {
                            request_height = suggested_height;
                        }
                    },
                    _ => {
                        unimplemented!();
                    }
                };
            },
            PositionType::Absolute | PositionType::Fixed => {
                match style.get_display() {
                    DisplayType::None => {
                        request_width = 0.;
                        request_height = 0.;
                    },
                    _ => {
                        request_width = 0.;
                        request_height = 0.;
                        let absolute_request_width = suggested_width; // FIXME calc it!
                        inline_position_status.reset(absolute_request_width);
                        if element.content().is_terminated() {
                            element.content_mut().suggest_size((suggested_size.0, 0.), inline_position_status, &*element.style());
                        } else {
                            for child in element.tree_node().iter_children() {
                                child.elem().suggest_size((suggested_size.0, 0.), inline_position_status);
                            }
                        }
                        inline_position_status.reset(absolute_request_width);
                    }
                };
            },
            _ => {
                unimplemented!();
            }
        }

        self.requested_size = (request_width, request_height);
        debug!("Suggested size for {} with ({}, {}), requested ({}, {})", element, suggested_size.0, suggested_size.1, self.requested_size.0, self.requested_size.1);
        self.requested_size
    }
    pub fn allocate_position(&mut self, is_dirty: bool, allocated_position: (f64, f64, f64, f64), element: &Element) -> (f64, f64, f64, f64) {
        if !is_dirty && allocated_position == self.allocated_position {
            return self.drawing_bounds
        }
        self.allocated_position = allocated_position;
        let mut current_height = 0.;
        let mut current_inline_height = 0.;
        self.drawing_bounds = allocated_position;
        if element.content().is_terminated() {
            let child_bounds = element.content().drawing_bounds();
            self.merge_drawing_bounds(&child_bounds);
        } else {
            for child in element.tree_node().iter_children() {
                let element = child.elem();
                let (requested_width, requested_height) = element.requested_size();
                let child_style = child.elem().style();

                match child_style.get_position() {
                    PositionType::Static => {
                        match child_style.get_display() {
                            DisplayType::None => {
                                /* do nothing */
                            },
                            DisplayType::Block => {
                                if current_inline_height > 0. {
                                    current_height += current_inline_height;
                                    current_inline_height = 0.;
                                }
                                let child_bounds = element.allocate_position((0., current_height, allocated_position.2, requested_height));
                                self.merge_drawing_bounds(&child_bounds);
                                current_height += requested_height;
                            },
                            DisplayType::Inline | DisplayType::InlineBlock => {
                                // the allocated height for inline nodes should be zero, so that drawing_bounds is empty for inline nodes themselves
                                let child_bounds = element.allocate_position((0., current_height, allocated_position.2, 0.));
                                self.merge_drawing_bounds(&child_bounds);
                                current_inline_height += requested_height;
                            },
                            _ => {
                                unimplemented!();
                            }
                        };
                    },
                    PositionType::Absolute => {
                        match child_style.get_display() {
                            DisplayType::None => {
                                /* do nothing */
                            },
                            _ => {
                                let left = if child_style.get_left() == DEFAULT_F64 { 0. } else { child_style.get_left() };
                                let top = if child_style.get_top() == DEFAULT_F64 { 0. } else { child_style.get_top() };
                                let child_bounds = element.allocate_position((left, top, requested_width, requested_height));
                                self.merge_drawing_bounds(&child_bounds);
                            }
                        };
                    },
                    _ => {
                        unimplemented!();
                    }
                }

            }
        }
        debug!("Allocated position for {} with {:?} drawing bounds {:?}", element, allocated_position, self.drawing_bounds);
        self.drawing_bounds
    }
}

// inline status

pub struct InlinePositionStatus {
    current_line_nodes: Vec<TreeNodeRc<Element>>,
    width: f64,
    used_height: f64,
    used_width: f64,
    line_height: f64, // total height
    baseline_offset: f64, // height above baseline
    last_required_line_height: f64,
    last_required_baseline_offset: f64,
}

impl InlinePositionStatus {
    pub fn new(width: f64) -> Self {
        Self {
            current_line_nodes: vec![],
            width: width,
            used_height: 0.,
            used_width: 0.,
            line_height: 0.,
            baseline_offset: 0.,
            last_required_line_height: 0.,
            last_required_baseline_offset: 0.,
        }
    }
    #[inline]
    pub fn height(&mut self) -> f64 {
        let height = self.used_height + self.line_height;
        height
    }
    pub fn reset(&mut self, width: f64) {
        self.current_line_nodes.truncate(0);
        self.width = width;
        self.used_height = 0.;
        self.used_width = 0.;
        self.line_height = 0.;
        self.baseline_offset = 0.;
    }
    pub fn append_node(&mut self, next_node: TreeNodeRc<Element>, required_line_height: f64, required_baseline_offset: f64) {
        self.last_required_line_height = required_line_height;
        self.last_required_baseline_offset = required_baseline_offset;
        if self.baseline_offset < required_baseline_offset {
            let bf = self.baseline_offset;
            self.line_height += required_baseline_offset - bf;
            self.baseline_offset = required_baseline_offset;
            self.adjust_baseline_offset(required_baseline_offset - bf);
        }
        if self.line_height < required_line_height {
            self.line_height = required_line_height;
        }
        self.current_line_nodes.push(next_node);
    }
    #[inline]
    pub fn line_height(&self) -> f64 { self.line_height }
    #[inline]
    pub fn baseline_offset(&self) -> f64 { self.baseline_offset }
    pub fn add_width(&mut self, width: f64, allow_line_wrap: bool) -> (f64, f64) {
        if self.used_width + width > self.width && self.used_width > 0. {
            if allow_line_wrap {
                self.line_wrap();
            }
        }
        let ret = (self.used_width, self.used_height + self.baseline_offset);
        self.used_width += width;
        ret
    }
    pub fn line_wrap(&mut self) {
        let last_node = self.current_line_nodes.pop().unwrap();
        self.current_line_nodes.truncate(0);
        self.used_width = 0.;
        self.used_height += self.line_height;
        self.line_height = 0.;
        self.baseline_offset = 0.;
        let lh = self.last_required_line_height;
        let bo = self.last_required_baseline_offset;
        self.append_node(last_node, lh, bo);
    }

    #[inline]
    fn adjust_baseline_offset(&mut self, add_offset: f64) {
        for node in self.current_line_nodes.iter_mut() {
            node.elem().content_mut().adjust_baseline_offset(add_offset);
        }
    }

}
