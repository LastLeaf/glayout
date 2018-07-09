#![macro_use]

mod style;
pub type ElementStyle = style::ElementStyle;
mod position_offset;
pub type PositionOffset = position_offset::PositionOffset;

mod empty_element;
pub type Empty = empty_element::Empty;
mod image_element;
pub type Image = image_element::Image;
mod text_element;
pub type Text = text_element::Text;

use std::rc::Rc;
use std::cell::{Cell, RefCell, Ref, RefMut};
use std::fmt;
use downcast_rs::Downcast;
use super::CanvasConfig;
use super::super::tree::{TreeElem, TreeNodeRc};

pub trait ElementContent: Downcast {
    fn name(&self) -> &'static str;
    fn associate_tree_node(&mut self, _node: TreeNodeRc<Element>) { }
    fn draw(&mut self, style: &ElementStyle, position_offset: &PositionOffset);
    fn suggest_size(&mut self, _suggested_size: (f64, f64), _style: &ElementStyle) -> (f64, f64) {
        (0., 0.)
    }
}

impl_downcast!(ElementContent);

pub struct Element {
    tree_node: Cell<Option<TreeNodeRc<Element>>>,
    dirty: Cell<bool>,
    style: RefCell<ElementStyle>,
    position_offset: RefCell<PositionOffset>,
    content: RefCell<Box<ElementContent>>,
}

impl Element {
    pub fn new(_cfg: &Rc<CanvasConfig>, content: Box<ElementContent>) -> Self {
        Element {
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
        ret
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
    pub fn draw(&self) {
        self.content.borrow_mut().draw(&*self.style(), &*self.position_offset());
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
        self.dirty.set(true);
        let tn = self.tree_node.replace(None);
        match tn.as_ref().unwrap().get_parent() {
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
    #[inline]
    pub fn get_requested_size(&self) -> (f64, f64) {
        self.position_offset.borrow().get_requested_size()
    }
    #[inline]
    pub fn suggest_size(&self, suggested_size: (f64, f64)) -> (f64, f64) {
        let is_dirty = self.is_dirty();
        self.position_offset.borrow_mut().suggest_size(is_dirty, suggested_size, self)
    }
    #[inline]
    pub fn allocate_position(&self, pos: (f64, f64, f64, f64)) {
        let is_dirty = self.clear_dirty();
        self.position_offset.borrow_mut().allocate_position(is_dirty, pos, self);
    }
    #[inline]
    pub fn update_position_offset(&self, suggested_size: (f64, f64)) {
        let requested_size = self.suggest_size(suggested_size);
        self.allocate_position((0., 0., requested_size.0, requested_size.1));
    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{name}", name = self.name())
    }
}

impl TreeElem for Element {
    fn associate_node(&self, node: TreeNodeRc<Element>) {
        self.tree_node.set(Some(node.clone()));
        self.content.borrow_mut().associate_tree_node(node);
    }
}

#[macro_export]
macro_rules! __element_children {
    ($cfg:expr, $v:ident, $t:ident, ) => {};
    ($cfg:expr, $v:ident, $t:ident, $k:ident = $a:expr; $($r:tt)*) => {
        $v.elem().style_mut().$k = $a;
        __element_children! ($cfg, $v, $t, $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, . $k:ident = $a:expr; $($r:tt)*) => {
        $v.elem().content_mut().downcast_mut::<$t>().unwrap().$k = $a;
        __element_children! ($cfg, $v, $t, $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, . $k:ident ( $($a:expr),* ); $($r:tt)*) => {
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
