#![macro_use]

use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use std::fmt::Debug;
use std::any::Any;
use std::rc::Rc;
use std::fmt;
use downcast_rs::Downcast;
use super::CanvasConfig;
use super::resource::DrawState;
use rc_forest::{ForestNodeContent, ForestNode, ForestNodeRc, ForestNodeSelf};

pub mod style;
pub use self::style::*;
mod positioning;
pub use self::positioning::{Position, Size, Point, Bounds};
use self::positioning::{PositionOffset, InlineAllocator};
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
    fn associate_element(&mut self, *mut Element) { }
    fn draw(&mut self, transform: &Transform);
    #[inline]
    fn suggest_size(&mut self, _suggested_size: Size, _inline_allocator: &mut InlineAllocator, _style: &ElementStyle) -> Size {
        Size::new(0., 0.)
    }
    #[inline]
    fn adjust_baseline_offset(&mut self, _add_offset: f64) {
        /* empty */
    }
    #[inline]
    fn adjust_text_align_offset(&mut self, _add_offset: f64) {
        /* empty */
    }
    fn drawing_bounds(&self) -> Bounds;
    fn is_under_point(&self, point: Point, transform: Transform) -> bool;
}

impl_downcast!(ElementContent);

pub struct Element {
    canvas_config: Rc<CanvasConfig>,
    tree_node: Option<ForestNodeSelf<Element>>,
    event_receiver: EventReceiver,
    dirty: bool,
    style: ElementStyle,
    position_offset: PositionOffset,
    draw_separate_tex: Cell<i32>,
    content: Box<ElementContent>,
}

impl Debug for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}({}) id=\"{}\" class=\"{}\"> @ {:p}", self.style().get_tag_name(), self.content.name(), self.style().get_id(), self.style().get_class(), self)
    }
}

impl Clone for Element {
    fn clone(&self) -> Self {
        Element {
            canvas_config: self.canvas_config.clone(),
            tree_node: None,
            event_receiver: EventReceiver::new(),
            dirty: true,
            style: ElementStyle::new(),
            position_offset: PositionOffset::new(),
            draw_separate_tex: Cell::new(-1),
            content: self.content.clone(),
        }
    }
}

impl Element {
    pub fn new(cfg: &Rc<CanvasConfig>, content: Box<ElementContent>) -> Self {
        Element {
            canvas_config: cfg.clone(),
            tree_node: None,
            event_receiver: EventReceiver::new(),
            dirty: true,
            style: ElementStyle::new(),
            position_offset: PositionOffset::new(),
            draw_separate_tex: Cell::new(-1),
            content,
        }
    }
    #[inline]
    pub fn rc(&self) -> ForestNodeRc<Element> {
        self.tree_node.as_ref().unwrap().rc()
    }
    #[inline]
    pub fn node(&self) -> &ForestNode<Element> {
        self.tree_node.clone().unwrap().deref_by(self)
    }
    #[inline]
    pub fn node_mut<'a>(&'a mut self) -> &'a mut ForestNode<Element> {
        self.tree_node.clone().unwrap().deref_mut_by(self)
    }
    #[inline]
    pub fn content(&self) -> &Box<ElementContent> {
        &self.content
    }
    #[inline]
    pub fn content_mut(&mut self) -> &mut Box<ElementContent> {
        &mut self.content
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        self.content.name()
    }

    #[inline]
    pub fn add_event_listener(&mut self, event_name: String, f: EventCallback) {
        self.event_receiver.add_listener(event_name, f);
    }
    #[inline]
    pub fn remove_event_listener(&mut self, event_name: String, f: EventCallback) {
        self.event_receiver.remove_listener(event_name, f);
    }
    #[inline]
    pub fn dispatch_event(&mut self, event_name: String, detail: Box<Any + 'static>, bubbles: bool) {
        let rc = self.node().rc();
        self.do_dispatch_event(event_name, &rc, &detail, bubbles);
    }
    fn do_dispatch_event(&mut self, event_name: String, target: &ForestNodeRc<Element>, detail: &Box<Any + 'static>, bubbles: bool) {
        // debug!("Dispatch {:?} event for {:?}", event_name, self);
        let rc = self.rc();
        let ev = Event::new(event_name.clone(), target, &rc, detail);
        ev.dispatch(self);
        if bubbles {
            match self.node_mut().parent_mut() {
                None => { },
                Some(node) => {
                    node.do_dispatch_event(event_name, target, detail, true);
                }
            }
        }
    }

    #[inline]
    pub fn tag_name(&mut self, tag_name: String) {
        self.style_mut().tag_name(tag_name);
    }
    #[inline]
    pub fn id(&mut self, id: String) {
        self.style_mut().id(id);
    }
    #[inline]
    pub fn class(&mut self, class_names: String) {
        self.style_mut().class(class_names);
    }
    #[inline]
    pub fn style(&self) -> &ElementStyle {
        &self.style
    }
    #[inline]
    pub fn style_mut(&mut self) -> &mut ElementStyle {
        &mut self.style
    }
    #[inline]
    pub fn style_inline_text(&mut self, text: &str) {
        self.style.inline_text(text)
    }

    #[inline]
    pub(crate) fn mark_layout_dirty(&mut self) {
        if self.dirty { return; }
        self.dirty = true;
        match self.node_mut().parent_mut() {
            None => { },
            Some(x) => x.mark_layout_dirty()
        }
    }
    #[inline]
    pub(crate) fn clear_layout_dirty(&mut self) -> bool {
        let ret = self.dirty;
        self.dirty = false;
        ret
    }
    #[inline]
    pub(crate) fn is_layout_dirty(&self) -> bool {
        self.dirty
    }
    #[inline]
    pub(crate) fn requested_size(&self) -> Size {
        self.position_offset.requested_size()
    }
    #[inline]
    pub(crate) fn drawing_bounds(&self) -> Bounds {
        self.position_offset.drawing_bounds()
    }
    #[inline]
    pub(crate) fn suggest_size(&mut self, suggested_size: Size, inline_allocator: &mut InlineAllocator, handle_absolute: bool) -> Size {
        let is_layout_dirty = self.is_layout_dirty();
        self.position_offset.suggest_size(is_layout_dirty, suggested_size, inline_allocator, handle_absolute)
    }
    #[inline]
    pub(crate) fn suggest_size_absolute(&mut self, relative_size: Size, inline_allocator: &mut InlineAllocator) {
        let is_layout_dirty = self.is_layout_dirty();
        self.position_offset.suggest_size_absolute(is_layout_dirty, relative_size, inline_allocator)
    }
    #[inline]
    pub(crate) fn allocate_position(&mut self, allocated_point: Point, relative_point: Point) -> Bounds {
        let is_layout_dirty = self.clear_layout_dirty();
        self.position_offset.allocate_position(is_layout_dirty, allocated_point, relative_point)
    }
    #[inline]
    pub(crate) fn dfs_update_position_offset(&mut self, suggested_size: Size) {
        let _requested_size = self.suggest_size(suggested_size, &mut InlineAllocator::new(), true);
        self.allocate_position(Point::new(0., 0.), Point::new(0., 0.));
    }

    #[inline]
    fn draw_rect(&mut self, color: (f32, f32, f32, f32), position: Position) {
        let rm = self.canvas_config.resource_manager();
        let mut rm = rm.borrow_mut();
        rm.set_draw_state(DrawState::new().color(color));
        debug!("Try drawing rect at {:?} colored {:?}", position, color);
        rm.request_draw(
            -2, true,
            0., 0., 1., 1.,
            position.into()
        );
    }
    #[inline]
    fn draw_background_color(&mut self, requested_size: Size, child_transform: &Transform) {
        let color = self.style.get_background_color();
        if color.3 > 0. {
            let position = Position::new(
                self.style.get_margin_left() + self.style.get_border_left_width(),
                self.style.get_margin_top() + self.style.get_border_top_width(),
                requested_size.width() - self.style.get_margin_left() - self.style.get_margin_right() - self.style.get_border_left_width() - self.style.get_border_right_width(),
                requested_size.height() - self.style.get_margin_top() - self.style.get_margin_bottom() - self.style.get_border_top_width() - self.style.get_border_bottom_width(),
            );
            self.draw_rect(color, child_transform.apply_to_position(&position));
        }
    }
    #[inline]
    fn draw_borders(&mut self, requested_size: Size, child_transform: &Transform) {
        if self.style.get_border_top_width() > 0. {
            let color = self.style.get_border_top_color();
            let position = Position::new(
                self.style.get_margin_left(),
                self.style.get_margin_top(),
                requested_size.width() - self.style.get_margin_left() - self.style.get_margin_right(),
                self.style.get_border_top_width(),
            );
            self.draw_rect(color, child_transform.apply_to_position(&position));
        }
        if self.style.get_border_bottom_width() > 0. {
            let color = self.style.get_border_bottom_color();
            let position = Position::new(
                self.style.get_margin_left(),
                requested_size.height() - self.style.get_margin_bottom() - self.style.get_border_bottom_width(),
                requested_size.width() - self.style.get_margin_left() - self.style.get_margin_right(),
                self.style.get_border_bottom_width(),
            );
            self.draw_rect(color, child_transform.apply_to_position(&position));
        }
        if self.style.get_border_left_width() > 0. {
            let color = self.style.get_border_left_color();
            let position = Position::new(
                self.style.get_margin_left(),
                self.style.get_margin_top() + self.style.get_border_top_width(),
                self.style.get_border_left_width(),
                requested_size.height() - self.style.get_margin_top() - self.style.get_margin_bottom() - self.style.get_border_top_width() - self.style.get_border_bottom_width(),
            );
            self.draw_rect(color, child_transform.apply_to_position(&position));
        }
        if self.style.get_border_right_width() > 0. {
            let color = self.style.get_border_right_color();
            let position = Position::new(
                requested_size.width() - self.style.get_margin_right() - self.style.get_border_right_width(),
                self.style.get_margin_top() + self.style.get_border_top_width(),
                self.style.get_border_right_width(),
                requested_size.height() - self.style.get_margin_top() - self.style.get_margin_bottom() - self.style.get_border_top_width() - self.style.get_border_bottom_width(),
            );
            self.draw_rect(color, child_transform.apply_to_position(&position));
        }
    }
    pub(crate) fn draw(&mut self, viewport: Position, mut transform: Transform) {
        if self.style.get_display() == style::DisplayType::None { return }
        // debug!("Drawing {:?}", self);
        let allocated_point = self.position_offset.allocated_point();
        let requested_size = self.position_offset.requested_size();
        let allocated_position = Position::from((allocated_point, requested_size));

        // check if drawing on separate tex is needed
        if self.style.get_opacity() < 1. && self.style.get_opacity() >= 0. {
            self.enable_draw_separate_tex()
        } else {
            self.disable_draw_separate_tex()
        }
        let tex_id = self.draw_separate_tex.get();
        let (drawing_tex_position, drawing_tex_offset) = if tex_id >= 0 {
            let drawing_bounds = self.style.transform_ref().apply_to_bounds(&self.position_offset.drawing_bounds());
            let drawing_tex_position = Position::new(0., 0., (drawing_bounds.width() + 1.).floor(), (drawing_bounds.height() + 1.).floor());
            // FIXME use drawing_bounds is incorrect because child's transform is not considered
            let rm = self.canvas_config.resource_manager();
            let mut rm = rm.borrow_mut();
            rm.bind_rendering_target(tex_id, drawing_tex_position.width() as i32, drawing_tex_position.height() as i32);
            (drawing_tex_position, Size::new(drawing_bounds.left(), drawing_bounds.top()))
        } else {
            (allocated_position, Size::new(0., 0.))
        };

        let child_transform = transform.mul_clone(Transform::new().offset(drawing_tex_position.left_top() - Point::new(0., 0.))).mul_clone(&self.style.transform_ref());

        // draw content and child
        self.draw_background_color(requested_size, &child_transform);
        self.draw_borders(requested_size, &child_transform);
        {
            self.content.draw(&child_transform);
            if !self.content.is_terminated() {
                let node = self.node_mut();
                for child in node.clone_children().iter() {
                    child.deref_mut_with(node).draw(viewport, child_transform);
                }
            }
        }

        // recover tex
        if tex_id >= 0 {
            let rm = self.canvas_config.resource_manager();
            let mut rm = rm.borrow_mut();
            rm.unbind_rendering_target();

            // set alpha
            let mut original_alpha = -1.;
            if self.style.get_opacity() < 1. && self.style.get_opacity() >= 0. {
                let mut ds = rm.draw_state();
                original_alpha = ds.get_alpha();
                rm.set_draw_state(ds.mul_alpha(self.style.get_opacity()));
            }

            rm.set_draw_state(DrawState::new().color(self.style.get_background_color()));
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
    fn get_node_under_point(&self, point: Point, mut transform: Transform) -> Option<ForestNodeRc<Element>> {
        if self.style().get_display() == style::DisplayType::None { return None }
        let position_offset = &self.position_offset;
        let allocated_point = position_offset.allocated_point();
        let self_transform = transform.mul_clone(Transform::new().offset(allocated_point.into())).mul_clone(&self.style().transform_ref());
        let drawing_bounds = self_transform.apply_to_bounds(&position_offset.drawing_bounds());
        // debug!("testing {:?} in bounds {:?}", (x, y), drawing_bounds);
        if !point.in_bounds(&drawing_bounds) {
            return None;
        }
        let content = &self.content;
        if content.is_terminated() {
            // debug!("testing {:?} in terminated {:?}", (x, y), content.name());
            if content.is_under_point(point, self_transform) {
                return Some(self.rc());
            }
        } else {
            let self_node = self.node();
            for child in self_node.iter().rev() {
                let child_match = child.deref_with(self_node).get_node_under_point(point, self_transform);
                if child_match.is_some() {
                    return child_match;
                }
            }
        }
        let allocated_position = self_transform.apply_to_position(&Position::from((Point::new(0., 0.), position_offset.requested_size())));
        // debug!("testing {:?} in allocated_position {:?}", (x, y), allocated_position);
        if point.in_position(&allocated_position) {
            return None;
        }
        Some(self.rc())
    }
    pub fn node_under_point(&self, point: Point) -> Option<ForestNodeRc<Element>> {
        self.get_node_under_point(point, Transform::new())
    }

    fn get_node_by_id(top: &ForestNode<Element>, node: &ForestNode<Element>, id: &str) -> Option<ForestNodeRc<Element>> {
        for child_rc in node.iter() {
            let child = child_rc.deref_with(top);
            if child.style().get_id() == id {
                return Some(child_rc.clone());
            }
            match Self::get_node_by_id(top, child, id) {
                Some(x) => {
                    return Some(x);
                },
                None => { }
            }
        }
        None
    }
    pub fn node_by_id(&self, id: &str) -> Option<ForestNodeRc<Element>> {
        if self.style().get_id() == id {
            return Some(self.rc());
        }
        Self::get_node_by_id(self.node(), self.node(), id)
    }
}


impl ForestNodeContent for Element {
    #[inline]
    fn associate_node(&mut self, node: ForestNodeSelf<Element>) {
        self.tree_node.replace(node);
        let self_ptr = self as *mut Self;
        self.style_mut().associate_element(self_ptr);
        self.position_offset.associate_element(self_ptr);
        self.content.associate_element(self_ptr);
    }
    #[inline]
    fn parent_node_changed(&mut self) {
        self.style_mut().parent_node_changed();
        match self.node_mut().parent_mut() {
            None => { },
            Some(parent_node) => {
                parent_node.mark_layout_dirty();
            }
        }
    }
}

impl Deref for Element {
    type Target = Box<ElementContent>;
    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl DerefMut for Element {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

// impl Drop for Element {
//     fn drop(&mut self) {
//         debug!("Drop element {:?}", self);
//     }
// }


#[macro_export]
macro_rules! __element_children {
    ($cfg:expr, $v:ident, $t:ident, ) => {};
    ($cfg:expr, $v:ident, $t:ident, @ $k:expr => $a:expr; $($r:tt)*) => {
        // event listeners
        $v.add_event_listener(String::from($k), Rc::new(RefCell::new($a)));
        __element_children! ($cfg, $v, $t, $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, class: $a:expr; $($r:tt)*) => {
        // inline styles
        $v.class($a.into());
        __element_children! ($cfg, $v, $t, $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, $k:ident : $a:expr; $($r:tt)*) => {
        // inline styles
        $v.style_mut().$k($a.into());
        __element_children! ($cfg, $v, $t, $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, $k:ident ( $($a:expr),* ); $($r:tt)*) => {
        // element content methods
        $v.content_mut().downcast_mut::<$t>().unwrap().$k($($a),*);
        __element_children! ($cfg, $v, $t, $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, $e:ident; $($r:tt)*) => {
        // child nodes (short form)
        __element_children! ($cfg, $v, $t, $e {}; $($r)*);
    };
    ($cfg:expr, $v:ident, $t:ident, $e:ident { $($c:tt)* }; $($r:tt)*) => {
        // child nodes
        let mut temp_element_child = __element_tree! ( $cfg, $v, $e { $($c)* });
        $v.append(temp_element_child);
        __element_children! ($cfg, $v, $t, $($r)*);
    }
}

#[macro_export]
macro_rules! __element_tree {
    ($node:expr, $cfg:expr, $e:ident) => {
        __element_tree! ($cfg, $e {})
    };
    ($cfg:expr, $node:expr, $e:ident { $($c:tt)* }) => {{
        let mut temp_content = Box::new($e::new($cfg));
        let mut temp_element = $node.create_another(Element::new($cfg, temp_content));
        {
            let mut _temp_element_inner = temp_element.deref_mut_with($node);
            __element_children! ($cfg, _temp_element_inner, $e, $($c)*);
        }
        temp_element
    }}
}

#[macro_export]
macro_rules! element {
    ($node:expr, $cfg:expr, $($c:tt)*) => {{
        __element_tree! ($cfg, $node, $($c)*)
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
