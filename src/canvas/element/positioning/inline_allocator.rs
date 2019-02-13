use rc_forest::{ForestNode, ForestNodeRc};
use super::super::Element;
use super::super::style::TextAlignType;
use super::{Point, Size};

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct InlineSize {
    size: Size,
    used_width: f64,
    line_height: f64,
    baseline_offset: f64,
}

impl InlineSize {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            used_width: 0.,
            line_height: 0.,
            baseline_offset: 0.,
        }
    }
    #[inline]
    pub fn size(&self) -> Size {
        self.size
    }
}

pub struct InlineAllocator {
    current_line_nodes: Vec<ForestNodeRc<Element>>,
    width: f64,
    text_align: TextAlignType,
    height: f64, // total height (excludes latest line)
    expected_width: f64, // the actual width used
    current_node_height: f64, // the height of latest node (excludes latest line)
    used_width: f64, // the occupied width for current line
    line_height: f64, // total height
    baseline_offset: f64, // height above baseline
    last_required_line_height: f64,
    last_required_baseline_offset: f64,
}

impl InlineAllocator {
    pub(crate) fn new() -> Self {
        Self {
            current_line_nodes: vec![],
            width: 0.,
            text_align: TextAlignType::Left,
            height: 0.,
            expected_width: 0.,
            current_node_height: 0.,
            used_width: 0.,
            line_height: 0.,
            baseline_offset: 0.,
            last_required_line_height: 0.,
            last_required_baseline_offset: 0.,
        }
    }
    pub(crate) fn reset(&mut self, current_node: &mut ForestNode<Element>, width: f64, text_align: TextAlignType) {
        self.width = width;
        self.text_align = text_align;
        if self.current_line_nodes.len() > 0 {
            self.apply_text_align(current_node);
            self.current_line_nodes.truncate(0);
            self.height = 0.;
            self.expected_width = 0.;
            self.current_node_height = 0.;
            self.used_width = 0.;
            self.line_height = 0.;
            self.baseline_offset = 0.;
            self.last_required_line_height = 0.;
            self.last_required_baseline_offset = 0.;
        }
    }
    #[inline]
    pub(crate) fn get_current_width(&self) -> f64 {
        self.expected_width
    }
    #[inline]
    pub(crate) fn get_current_height(&self) -> f64 {
        self.height + if self.used_width > 0. { self.line_height } else { 0. }
    }
    pub(crate) fn start_node(&mut self, next_node: &mut ForestNode<Element>, required_line_height: f64, required_baseline_offset: f64) {
        self.last_required_line_height = required_line_height;
        self.last_required_baseline_offset = required_baseline_offset;
        if self.baseline_offset < required_baseline_offset {
            let bf = self.baseline_offset;
            self.line_height += required_baseline_offset - bf;
            self.baseline_offset = required_baseline_offset;
            self.adjust_baseline_offset(next_node, required_baseline_offset - bf);
        }
        if self.line_height < required_line_height {
            self.line_height = required_line_height;
        }
        self.current_node_height = 0.;
        self.current_line_nodes.push(next_node.rc());
    }
    pub(crate) fn end_node(&mut self) -> InlineSize {
        // TODO use this to prevent re-cal
        let height = self.current_node_height + if self.used_width > 0. { self.line_height } else { 0. };
        InlineSize {
            size: Size::new(self.width, height),
            used_width: self.used_width,
            line_height: if self.used_width > 0. { self.line_height } else { 0. },
            baseline_offset: if self.used_width > 0. { self.baseline_offset } else { 0. },
        }
    }
    pub(crate) fn add_width(&mut self, current_node: &mut ForestNode<Element>, width: f64, allow_line_wrap: bool) -> Point {
        if self.used_width + width > self.width && self.used_width > 0. {
            if allow_line_wrap {
                self.line_wrap(current_node);
            }
        }
        let ret = Point::new(self.used_width, self.current_node_height + self.baseline_offset);
        self.used_width += width;
        if self.expected_width < self.used_width { self.expected_width = self.used_width }
        ret
    }
    fn apply_text_align(&mut self, current_node: &mut ForestNode<Element>) {
        match self.text_align {
            TextAlignType::Left => { },
            TextAlignType::Center => {
                let d = self.width - self.used_width;
                self.adjust_text_align_offset(current_node, d / 2.);
            },
            TextAlignType::Right => {
                let d = self.width - self.used_width;
                self.adjust_text_align_offset(current_node, d);
            },
        };
    }
    pub(crate) fn line_wrap(&mut self, current_node: &mut ForestNode<Element>) {
        self.apply_text_align(current_node);
        let last_node = self.current_line_nodes.pop().unwrap();
        self.current_line_nodes.truncate(0);
        self.current_line_nodes.push(last_node);
        self.height += self.line_height;
        self.current_node_height += self.line_height;
        self.used_width = 0.;
        self.line_height = self.last_required_line_height;
        self.baseline_offset = self.last_required_baseline_offset;
    }

    #[inline]
    fn adjust_baseline_offset(&mut self, current_node: &mut ForestNode<Element>, add_offset: f64) {
        for node in self.current_line_nodes.iter_mut() {
            node.deref_mut_with(current_node).adjust_baseline_offset(add_offset); // TODO
        }
    }
    #[inline]
    fn adjust_text_align_offset(&mut self, current_node: &mut ForestNode<Element>, add_offset: f64) {
        for node in self.current_line_nodes.iter_mut() {
            node.deref_mut_with(current_node).adjust_text_align_offset(add_offset); // TODO
        }
    }

}