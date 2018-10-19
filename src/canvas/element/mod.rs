#![macro_use]

pub mod style;
pub type ElementStyle = style::ElementStyle;
mod position_offset;
pub type PositionOffset = position_offset::PositionOffset;
pub type InlinePositionStatus = position_offset::InlinePositionStatus;
mod transform;
pub type Transform = transform::Transform;

mod empty_element;
pub type Empty = empty_element::Empty;
mod image_element;
pub type Image = image_element::Image;
pub type ImageLoader = image_element::ImageLoader;
mod text_element;
pub type Text = text_element::Text;

use std::rc::Rc;
use std::cell::{Cell, RefCell, Ref, RefMut};
use std::fmt;
use downcast_rs::Downcast;
use super::CanvasConfig;
use super::resource::DrawState;
use super::super::tree::{TreeElem, TreeNodeRc, TreeNodeWeak, TreeNodeSearchType};

pub trait ElementContent: Downcast {
    fn name(&self) -> &'static str;
    fn is_terminated(&self) -> bool;
    #[inline]
    fn associate_tree_node(&mut self, _node: TreeNodeWeak<Element>) { }
    fn draw(&mut self, style: &ElementStyle, transform: &Transform);
    #[inline]
    fn suggest_size(&mut self, _suggested_size: (f64, f64), _inline_position_status: &mut InlinePositionStatus, _style: &ElementStyle) -> (f64, f64) {
        (0., 0.)
    }
    #[inline]
    fn adjust_baseline_offset(&mut self, _add_offset: f64) {
        /* empty */
    }
    fn drawing_bounds(&self) -> (f64, f64, f64, f64);
    fn is_under_point(&self, x: f64, y: f64, transform: Transform) -> bool;
}

impl_downcast!(ElementContent);

pub struct Element {
    canvas_config: Rc<CanvasConfig>,
    tree_node: Cell<Option<TreeNodeWeak<Element>>>,
    dirty: Cell<bool>,
    style: RefCell<ElementStyle>,
    position_offset: RefCell<PositionOffset>,
    content: RefCell<Box<ElementContent>>,
}

impl Element {
    pub fn new(cfg: &Rc<CanvasConfig>, content: Box<ElementContent>) -> Self {
        Element {
            canvas_config: cfg.clone(),
            tree_node: Cell::new(None),
            dirty: Cell::new(true),
            style: RefCell::new(ElementStyle::new()),
            position_offset: RefCell::new(PositionOffset::new()),
            content: RefCell::new(content),
        }
    }
    #[inline]
    pub fn name(&self) -> &'static str {
        self.content.borrow().name()
    }
    #[inline]
    pub fn tree_node(&self) -> TreeNodeRc<Element> {
        let tn = self.tree_node.replace(None);
        let ret = tn.clone().unwrap();
        self.tree_node.replace(tn);
        ret.upgrade().unwrap()
    }
    #[inline]
    pub fn style(&self) -> Ref<ElementStyle> {
        self.style.borrow()
    }
    #[inline]
    pub fn style_mut(&self) -> RefMut<ElementStyle> {
        self.style.borrow_mut()
    }
    #[inline]
    pub fn position_offset(&self) -> Ref<PositionOffset> {
        self.position_offset.borrow()
    }
    #[inline]
    pub fn position_offset_mut(&self) -> RefMut<PositionOffset> {
        self.position_offset.borrow_mut()
    }

    #[inline]
    pub fn content(&self) -> Ref<Box<ElementContent>> {
        self.content.borrow()
    }
    #[inline]
    pub fn content_mut(&self) -> RefMut<Box<ElementContent>> {
        self.content.borrow_mut()
    }

    #[inline]
    pub fn mark_dirty(&self) {
        if self.dirty.replace(true) { return; }
        let tn = self.tree_node.replace(None);
        match tn.as_ref().unwrap().upgrade().unwrap().parent() {
            None => { },
            Some(ref x) => {
                x.elem().mark_dirty();
            }
        };
        self.tree_node.replace(tn);
    }
    #[inline]
    pub fn clear_dirty(&self) -> bool {
        self.dirty.replace(false)
    }
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty.get()
    }
    fn spread_dirty(&self) {
        // for dirty inline nodes, spread dirty to all inline nodes beside it
        let mut pending_inline_nodes: Vec<TreeNodeRc<Self>> = vec![];
        let mut inline_dirty = false;
        self.tree_node().dfs(TreeNodeSearchType::ChildrenLast, &mut |n| {
            let display = n.elem().style().get_display();
            match display {
                style::DisplayType::Inline | style::DisplayType::InlineBlock => {
                    if n.elem().dirty.get() {
                        if !inline_dirty {
                            inline_dirty = true;
                            for n in pending_inline_nodes.iter() {
                                n.elem().dirty.set(true);
                            }
                            pending_inline_nodes.truncate(0);
                        }
                    } else {
                        if inline_dirty {
                            n.elem().dirty.set(true);
                        } else {
                            pending_inline_nodes.push(n.clone());
                        }
                    }
                },
                _ => {
                    pending_inline_nodes.truncate(0);
                }
            }
            true
        });
    }
    #[inline]
    pub fn requested_size(&self) -> (f64, f64) {
        self.position_offset.borrow().requested_size()
    }
    #[inline]
    pub fn suggest_size(&self, suggested_size: (f64, f64), inline_position_status: &mut InlinePositionStatus) -> (f64, f64) {
        let is_dirty = self.is_dirty();
        self.position_offset.borrow_mut().suggest_size(is_dirty, suggested_size, inline_position_status, self)
    }
    #[inline]
    pub fn allocate_position(&self, pos: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
        let is_dirty = self.clear_dirty();
        self.position_offset.borrow_mut().allocate_position(is_dirty, pos, self)
    }
    #[inline]
    pub fn dfs_update_position_offset(&self, suggested_size: (f64, f64)) {
        self.spread_dirty();
        let requested_size = self.suggest_size(suggested_size, &mut InlinePositionStatus::new(suggested_size.0));
        self.allocate_position((0., 0., suggested_size.0, requested_size.1));
    }

    pub fn draw(&self, viewport: (f64, f64, f64, f64), mut transform: Transform) {
        let style = self.style();
        if style.get_display() == style::DisplayType::None { return }

        // set alpha
        // TODO gives a correct alpha impl
        let mut original_alpha = -1.;
        if style.get_opacity() < 1. && style.get_opacity() >= 0. {
            let rm = self.canvas_config.resource_manager();
            let mut rm = rm.borrow_mut();
            let mut ds = rm.draw_state();
            original_alpha = ds.get_alpha();
            rm.set_draw_state(ds.mul_alpha(style.get_opacity()));
        }

        let position_offset = self.position_offset();
        let allocated_position = position_offset.allocated_position();
        let child_transform = transform.mul_clone(Transform::new().offset(allocated_position.0, allocated_position.1)).mul_clone(&style.transform_ref());

        // draw background color
        let bg_color = style.get_background_color();
        if bg_color.0 >= 0. {
            let rm = self.canvas_config.resource_manager();
            let mut rm = rm.borrow_mut();
            rm.set_draw_state(DrawState::new().color(bg_color));
            // debug!("Try drawing rect at {:?} colored {:?}", child_transform.apply_to_position(&(0., 0., allocated_position.2, allocated_position.3)), bg_color);
            rm.request_draw(
                -2, true,
                0., 0., 1., 1.,
                child_transform.apply_to_position(&(0., 0., allocated_position.2, allocated_position.3))
            );
        }

        // draw content and child
        let mut content = self.content.borrow_mut();
        content.draw(&*self.style(), &child_transform);
        if !content.is_terminated() {
            for child in self.tree_node().iter_children() {
                child.elem().draw(viewport, child_transform);
            }
        }

        // recover alpha
        if original_alpha >= 0. {
            let rm = self.canvas_config.resource_manager();
            let mut rm = rm.borrow_mut();
            rm.set_draw_state(DrawState::new().alpha(original_alpha));
        }
    }

    fn get_node_under_point(&self, x: f64, y: f64, mut transform: Transform) -> Option<TreeNodeRc<Element>> {
        if self.style().get_display() == style::DisplayType::None { return None }
        let position_offset = self.position_offset();
        let allocated_position = position_offset.allocated_position();
        let child_transform = transform.mul_clone(Transform::new().offset(allocated_position.0, allocated_position.1)).mul_clone(&self.style().transform_ref());
        let drawing_bounds = child_transform.apply_to_bounds(&position_offset.drawing_bounds());
        debug!("testing {:?} in bounds {:?}", (x, y), drawing_bounds);
        if x < drawing_bounds.0 || x >= drawing_bounds.2 || y < drawing_bounds.1 || y >= drawing_bounds.3 {
            return None;
        }
        let content = self.content.borrow_mut();
        if content.is_terminated() {
            // debug!("testing {:?} in terminated {:?}", (x, y), content.name());
            if content.is_under_point(x, y, child_transform) {
                return Some(self.tree_node());
            }
        } else {
            for child in self.tree_node().iter_children().rev() {
                let child_match = child.elem().get_node_under_point(x, y, child_transform);
                if child_match.is_some() {
                    return child_match;
                }
            }
        }
        let allocated_position = position_offset.allocated_position();
        let allocated_position = child_transform.apply_to_position(&(0., 0., allocated_position.2, allocated_position.3));
        // debug!("testing {:?} in allocated_position {:?}", (x, y), allocated_position);
        if x < allocated_position.0 || x >= allocated_position.0 + allocated_position.2 || y < allocated_position.1 || y >= allocated_position.1 + allocated_position.3 {
            return None;
        }
        Some(self.tree_node())
    }
    pub fn node_under_point(&self, (x, y): (f64, f64)) -> Option<TreeNodeRc<Element>> {
        self.get_node_under_point(x, y, Transform::new())
    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{name}", name = self.name())
    }
}

impl TreeElem for Element {
    #[inline]
    fn associate_node(&self, node: TreeNodeWeak<Element>) {
        self.tree_node.set(Some(node.clone()));
        self.style_mut().associate_tree_node(node.clone());
        self.content.borrow_mut().associate_tree_node(node);
    }
    #[inline]
    fn parent_node_changed(&self, parent_node: Option<TreeNodeRc<Element>>) {
        self.style_mut().parent_node_changed(parent_node.clone());
        if parent_node.is_some() {
            parent_node.unwrap().elem().mark_dirty();
        }
    }
}

#[macro_export]
macro_rules! __element_children {
    ($cfg:expr, $v:ident, $t:ident, ) => {};
    ($cfg:expr, $v:ident, $t:ident, $k:ident : $a:expr; $($r:tt)*) => {
        $v.elem().style_mut().$k($a);
        __element_children! ($cfg, $v, $t, $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, $k:ident ( $($a:expr),* ); $($r:tt)*) => {
        $v.elem().content_mut().downcast_mut::<$t>().unwrap().$k($($a),*);
        __element_children! ($cfg, $v, $t, $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, $e:ident; $($r:tt)*) => {
        __element_children! ($cfg, $v, $t, $e {}; $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, $e:ident { $($c:tt)* }; $($r:tt)*) => {
        let mut temp_element_child = __element_tree! ( $cfg, $e { $($c)* });
        $v.append(temp_element_child);
        __element_children! ($cfg, $v, $t, $($r)*);
    }
}

#[macro_export]
macro_rules! __element_tree {
    ($cfg:expr, $e:ident) => {
        __element_tree! ($cfg, $e {})
    };
    ($cfg:expr, $e:ident { $($c:tt)* }) => {{
        let mut temp_content = Box::new($e::new($cfg));
        let mut temp_element = $crate::tree::TreeNodeRc::new(Element::new($cfg, temp_content));
        {
            let mut _temp_element_inner = temp_element.clone();
            __element_children! ($cfg, _temp_element_inner, $e, $($c)*);
        }
        temp_element
    }}
}

#[macro_export]
macro_rules! element {
    ([$cfg:expr] $($c:tt)*) => {{
        __element_tree! ($cfg, $($c)*)
    }}
}