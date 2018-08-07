use super::super::super::tree::TreeNodeRc;
use super::style::{DisplayType};
use super::Element;

// position offset

#[derive(Default, Debug)]
pub struct PositionOffset {
    suggested_size: (f64, f64),
    requested_size: (f64, f64),
    allocated_position: (f64, f64, f64, f64),
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

    pub fn suggest_size(&mut self, is_dirty: bool, suggested_size: (f64, f64), inline_position_status: &mut InlinePositionStatus, element: &Element) -> (f64, f64) {
        if !is_dirty && suggested_size == self.suggested_size {
            // TODO is an inline node is dirty, then all inline nodes beside it are dirty
            return self.requested_size
        }
        self.suggested_size = suggested_size;
        let request_width;
        let mut request_height = 0.; // for inline nodes, request_height is the added height while appending the node
        let style = element.style();
        match style.display {
            DisplayType::Block => {
                request_width = suggested_size.0;
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
            },
            DisplayType::Inline | DisplayType::InlineBlock => {
                request_width = suggested_size.0;
                if element.content().is_terminated() {
                    let (_, h) = element.content_mut().suggest_size((suggested_size.0, 0.), inline_position_status, &*element.style());
                    request_height += h;
                } else {
                    for child in element.tree_node().iter_children() {
                        let (_, h) = child.elem().suggest_size((suggested_size.0, 0.), inline_position_status);
                        request_height += h;
                    }
                }
            },
            _ => {
                unimplemented!();
            }
        };
        self.requested_size = (request_width, request_height);
        debug!("Suggested size for {} with ({}, {}), requested ({}, {})", element, suggested_size.0, suggested_size.1, self.requested_size.0, self.requested_size.1);
        self.requested_size
    }
    pub fn allocate_position(&mut self, is_dirty: bool, allocated_position: (f64, f64, f64, f64), element: &Element) {
        if !is_dirty && allocated_position == self.allocated_position {
            return
        }
        self.allocated_position = allocated_position;
        let mut current_height = 0.;
        let mut current_inline_height = 0.;
        if element.content().is_terminated() {
            /* do nothing */
        } else {
            for child in element.tree_node().iter_children() {
                let element = child.elem();
                let (_, h) = element.requested_size();
                let child_display = child.elem().style().display;
                match child_display {
                    DisplayType::Block => {
                        if current_inline_height > 0. {
                            current_height += current_inline_height;
                            current_inline_height = 0.;
                        }
                        element.allocate_position((0., current_height, allocated_position.2, h));
                        current_height += h;
                    },
                    DisplayType::Inline | DisplayType::InlineBlock => {
                        element.allocate_position((0., current_height, allocated_position.2, 0.));
                        current_inline_height += h;
                    },
                    _ => {
                        unimplemented!();
                    }
                };
            }
        }
        debug!("Allocated position for {} with ({}, {}) size ({}, {})", element, allocated_position.0, allocated_position.1, allocated_position.2, allocated_position.3);
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
    #[inline]
    pub fn reset(&mut self, width: f64) {
        self.current_line_nodes.truncate(0);
        self.width = width;
        self.used_width = 0.;
        self.line_height = 0.;
        self.baseline_offset = 0.;
    }
    #[inline]
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
    #[inline]
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
    #[inline]
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
