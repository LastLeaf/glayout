#![macro_use]

use std::fmt::Debug;
use std::any::Any;
use std::rc::Rc;
use std::cell::{Cell, RefCell, Ref, RefMut};
use std::fmt;
use downcast_rs::Downcast;
use super::CanvasConfig;
use super::resource::DrawState;
use super::super::tree::{TreeElem, TreeNodeRc, TreeNodeWeak};

pub mod style;
pub use self::style::ElementStyle;
mod positioning;
pub use self::positioning::{PositionOffset, Position, Size, Point, Bounds, InlineAllocator};
mod transform;
pub use self::transform::Transform;

mod empty_element;
pub use self::empty_element::Empty;
mod image_element;
pub use self::image_element::{Image, ImageLoader, ImageLoaderStatus};
mod text_element;
pub use self::text_element::Text;

mod event;
pub use self::event::{Event, EventReceiver, EventCallback};

pub trait ElementContent: Downcast {
    fn name(&self) -> &'static str;
    fn is_terminated(&self) -> bool;
    fn clone(&self) -> Box<ElementContent>;
    #[inline]
    fn associate_tree_node(&mut self, _node: TreeNodeWeak<Element>) { }
    fn draw(&mut self, style: &ElementStyle, transform: &Transform);
    #[inline]
    fn suggest_size(&mut self, _suggested_size: Size, _inline_allocator: &mut InlineAllocator, _style: &ElementStyle) -> Size {
        Size::new(0., 0.)
    }
    #[inline]
    fn adjust_baseline_offset(&mut self, _add_offset: f64) {
        /* empty */
    }
    fn drawing_bounds(&self) -> Bounds;
    fn is_under_point(&self, point: Point, transform: Transform) -> bool;
}

impl_downcast!(ElementContent);

pub struct Element {
    canvas_config: Rc<CanvasConfig>,
    tree_node: Cell<Option<TreeNodeWeak<Element>>>,
    event_receiver: RefCell<EventReceiver>,
    dirty: Cell<bool>,
    style: RefCell<ElementStyle>,
    position_offset: RefCell<PositionOffset>,
    draw_separate_tex: Cell<i32>,
    content: RefCell<Box<ElementContent>>,
}

impl Debug for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{} id={:?}>", self.name(), self.style().get_id())
    }
}

impl Clone for Element {
    fn clone(&self) -> Self {
        Element {
            canvas_config: self.canvas_config.clone(),
            tree_node: Cell::new(None),
            event_receiver: RefCell::new(EventReceiver::new()),
            dirty: Cell::new(true),
            style: RefCell::new(ElementStyle::new()),
            position_offset: RefCell::new(PositionOffset::new()),
            draw_separate_tex: Cell::new(-1),
            content: RefCell::new(self.content.borrow().clone()),
        }
    }
}

impl Element {
    pub fn new(cfg: &Rc<CanvasConfig>, content: Box<ElementContent>) -> Self {
        Element {
            canvas_config: cfg.clone(),
            tree_node: Cell::new(None),
            event_receiver: RefCell::new(EventReceiver::new()),
            dirty: Cell::new(true),
            style: RefCell::new(ElementStyle::new()),
            position_offset: RefCell::new(PositionOffset::new()),
            draw_separate_tex: Cell::new(-1),
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
    pub fn add_event_listener(&self, event_name: String, f: EventCallback) {
        self.event_receiver.borrow_mut().add_listener(event_name, f);
    }
    #[inline]
    pub fn remove_event_listener(&self, event_name: String, f: EventCallback) {
        self.event_receiver.borrow_mut().remove_listener(event_name, f);
    }
    #[inline]
    pub fn dispatch_event(&self, event_name: String, detail: Box<Any + 'static>, bubbles: bool) {
        self.do_dispatch_event(event_name, &detail, bubbles, self.tree_node().clone())
    }
    fn do_dispatch_event(&self, event_name: String, detail: &Box<Any + 'static>, bubbles: bool, target: TreeNodeRc<Element>) {
        // debug!("Dispatch {:?} event for {:?}", event_name, self);
        self.event_receiver.borrow().new_event(event_name.clone(), target.clone(), self.tree_node().clone(), detail);
        if bubbles {
            match self.tree_node().parent() {
                None => { },
                Some(node) => {
                    node.elem().do_dispatch_event(event_name, detail, true, target);
                }
            }
        }
    }

    #[inline]
    pub fn tag_name(&self, tag_name: String) {
        self.style_mut().tag_name(tag_name);
    }
    #[inline]
    pub fn id(&self, id: String) {
        self.style_mut().id(id);
    }
    #[inline]
    pub fn class(&self, class_names: String) {
        self.style_mut().class(class_names);
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
    pub fn style_inline_text(&self, text: &str) {
        self.style.borrow_mut().inline_text(text)
    }
    #[inline]
    pub(crate) fn position_offset(&self) -> Ref<PositionOffset> {
        self.position_offset.borrow()
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
    pub(crate) fn mark_dirty(&self) {
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
    pub(crate) fn clear_dirty(&self) -> bool {
        self.dirty.replace(false)
    }
    #[inline]
    pub(crate) fn is_dirty(&self) -> bool {
        self.dirty.get()
    }
    #[inline]
    pub(crate) fn requested_size(&self) -> Size {
        self.position_offset.borrow().requested_size()
    }
    #[inline]
    pub(crate) fn suggest_size(&self, suggested_size: Size, inline_allocator: &mut InlineAllocator) -> Size {
        let is_dirty = self.is_dirty();
        self.position_offset.borrow_mut().suggest_size(is_dirty, suggested_size, inline_allocator, self)
    }
    #[inline]
    pub(crate) fn allocate_position(&self, pos: Position) -> Bounds {
        let is_dirty = self.clear_dirty();
        self.position_offset.borrow_mut().allocate_position(is_dirty, pos, self)
    }
    #[inline]
    pub(crate) fn dfs_update_position_offset(&self, suggested_size: Size) {
        let requested_size = self.suggest_size(suggested_size, &mut InlineAllocator::new(suggested_size.width(), style::TextAlignType::Left));
        self.allocate_position(Position::new(0., 0., suggested_size.width(), requested_size.height()));
    }

    pub(crate) fn draw(&self, viewport: Position, mut transform: Transform) {
        let style = self.style();
        if style.get_display() == style::DisplayType::None { return }
        let position_offset = self.position_offset();
        let allocated_position = position_offset.allocated_position();

        // check if drawing on separate tex is needed
        if style.get_opacity() < 1. && style.get_opacity() >= 0. {
            self.enable_draw_separate_tex()
        } else {
            self.disable_draw_separate_tex()
        }
        let tex_id = self.draw_separate_tex.get();
        let (drawing_tex_position, drawing_tex_offset) = if tex_id >= 0 {
            let drawing_bounds = style.transform_ref().apply_to_bounds(&position_offset.drawing_bounds());
            let drawing_tex_position = Position::new(0., 0., (drawing_bounds.width() + 1.).floor(), (drawing_bounds.height() + 1.).floor());
            // FIXME use drawing_bounds is incorrect because child's transform is not considered
            let rm = self.canvas_config.resource_manager();
            let mut rm = rm.borrow_mut();
            rm.bind_rendering_target(tex_id, drawing_tex_position.width() as i32, drawing_tex_position.height() as i32);
            (drawing_tex_position, Size::new(drawing_bounds.left(), drawing_bounds.top()))
        } else {
            (allocated_position, Size::new(0., 0.))
        };

        let child_transform = transform.mul_clone(Transform::new().offset(drawing_tex_position.left_top() - Point::new(0., 0.))).mul_clone(&style.transform_ref());

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
                child_transform.apply_to_position(&Position::new(0., 0., allocated_position.width(), allocated_position.height())).into()
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

        // recover tex
        if tex_id >= 0 {
            let rm = self.canvas_config.resource_manager();
            let mut rm = rm.borrow_mut();
            rm.unbind_rendering_target();

            // set alpha
            let mut original_alpha = -1.;
            if style.get_opacity() < 1. && style.get_opacity() >= 0. {
                let mut ds = rm.draw_state();
                original_alpha = ds.get_alpha();
                rm.set_draw_state(ds.mul_alpha(style.get_opacity()));
            }

            rm.set_draw_state(DrawState::new().color(bg_color));
            rm.request_draw(
                tex_id, false,
                0., 0., 1., 1.,
                (allocated_position.left() + drawing_tex_offset.width(), allocated_position.top() + drawing_tex_offset.height(), drawing_tex_position.width(), drawing_tex_position.height())
            );

            // recover alpha
            if original_alpha >= 0. {
                rm.set_draw_state(DrawState::new().alpha(original_alpha));
            }
        }
    }
    #[inline]
    pub(crate) fn enable_draw_separate_tex(&self) {
        if self.draw_separate_tex.get() != -1 { return };
        let rm = self.canvas_config.resource_manager();
        let tex_id = rm.borrow_mut().alloc_tex_id();
        lib!(tex_create_empty(self.canvas_config.index, tex_id, 0, 0));
        self.draw_separate_tex.set(tex_id);
    }
    #[inline]
    pub(crate) fn disable_draw_separate_tex(&self) {
        if self.draw_separate_tex.get() == -1 { return };
        let tex_id = self.draw_separate_tex.replace(-1);
        let rm = self.canvas_config.resource_manager();
        lib!(tex_delete(self.canvas_config.index, tex_id));
        rm.borrow_mut().free_tex_id(tex_id);
    }

    // find the node under point
    // TODO check the transform correctness
    fn get_node_under_point(&self, point: Point, mut transform: Transform) -> Option<TreeNodeRc<Element>> {
        if self.style().get_display() == style::DisplayType::None { return None }
        let position_offset = self.position_offset();
        let allocated_position = position_offset.allocated_position();
        let child_transform = transform.mul_clone(Transform::new().offset(allocated_position.left_top() - Point::new(0., 0.))).mul_clone(&self.style().transform_ref());
        let drawing_bounds = child_transform.apply_to_bounds(&position_offset.drawing_bounds());
        // debug!("testing {:?} in bounds {:?}", (x, y), drawing_bounds);
        if !point.in_bounds(&drawing_bounds) {
            return None;
        }
        let content = self.content.borrow_mut();
        if content.is_terminated() {
            // debug!("testing {:?} in terminated {:?}", (x, y), content.name());
            if content.is_under_point(point, child_transform) {
                return Some(self.tree_node());
            }
        } else {
            for child in self.tree_node().iter_children().rev() {
                let child_match = child.elem().get_node_under_point(point, child_transform);
                if child_match.is_some() {
                    return child_match;
                }
            }
        }
        let allocated_position = position_offset.allocated_position();
        let allocated_position = child_transform.apply_to_position(&Position::new(0., 0., allocated_position.width(), allocated_position.height()));
        // debug!("testing {:?} in allocated_position {:?}", (x, y), allocated_position);
        if point.in_position(&allocated_position) {
            return None;
        }
        Some(self.tree_node())
    }
    pub fn node_under_point(&self, point: Point) -> Option<TreeNodeRc<Element>> {
        self.get_node_under_point(point, Transform::new())
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
    ($cfg:expr, $v:ident, $t:ident, @ $k:expr => $a:expr; $($r:tt)*) => {
        // event listeners
        $v.elem().add_event_listener(String::from($k), Rc::new(RefCell::new($a)));
        __element_children! ($cfg, $v, $t, $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, class: $a:expr; $($r:tt)*) => {
        // inline styles
        $v.elem().class($a.into());
        __element_children! ($cfg, $v, $t, $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, $k:ident : $a:expr; $($r:tt)*) => {
        // inline styles
        $v.elem().style_mut().$k($a.into());
        __element_children! ($cfg, $v, $t, $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, $k:ident ( $($a:expr),* ); $($r:tt)*) => {
        // element content methods
        $v.elem().content_mut().downcast_mut::<$t>().unwrap().$k($($a),*);
        __element_children! ($cfg, $v, $t, $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, $e:ident; $($r:tt)*) => {
        // child nodes (short form)
        __element_children! ($cfg, $v, $t, $e {}; $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, $e:ident { $($c:tt)* }; $($r:tt)*) => {
        // child nodes
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
    ($cfg:expr, $($c:tt)*) => {{
        __element_tree! ($cfg, $($c)*)
    }}
}

#[macro_export]
macro_rules! __element_class_rule {
    ($c:expr, ) => {};
    ($c:expr, $k:ident : $a:expr; $($r:tt)*) => {
        $c.add_rule(StyleName::$k, Box::new($a));
        __element_class_rule! ($c, $($r)*);
    };
}

#[macro_export]
macro_rules! element_class {
    ($($r:tt)*) => {{
        let mut c = ::std::rc::Rc::new(ElementClass::new());
        {
            let c = ::std::rc::Rc::get_mut(&mut c).unwrap();
            __element_class_rule! (c, $($r)*);
        };
        c
    }}
}
