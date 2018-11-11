use std::rc::Rc;
use std::cell::RefCell;
use super::super::super::tree::{TreeNodeRc, TreeElem};
use super::super::super::canvas::{Canvas, CanvasContext, CanvasConfig};
use super::super::super::canvas::element::{Element, Empty, Text, Image};

// canvas
pub extern "C" fn canvas_create(index: i32) -> *mut Canvas {
    let box_canvas = Box::new(Canvas::new(index));
    Box::into_raw(box_canvas)
}
pub extern "C" fn canvas_destroy(canvas: *mut Canvas) {
    unsafe { Box::from_raw(canvas) };
}
pub extern "C" fn canvas_get_context(canvas: *mut Canvas) -> *const RefCell<CanvasContext> {
    let canvas = unsafe { Box::from_raw(canvas) };
    let ret = Rc::into_raw(canvas.context());
    Box::into_raw(canvas);
    ret
}

// canvas context
#[inline]
fn canvas_context_from_pointer(context: *const RefCell<CanvasContext>) -> Rc<RefCell<CanvasContext>> {
    let ctx_ori = unsafe { Rc::from_raw(context) };
    let ctx = ctx_ori.clone();
    Rc::into_raw(ctx_ori);
    ctx
}
pub extern "C" fn canvas_context_set_canvas_size(context: *const RefCell<CanvasContext>, w: i32, h: i32, pixel_ratio: f64) {
    let ctx = canvas_context_from_pointer(context);
    ctx.borrow_mut().set_canvas_size(w, h, pixel_ratio);
}
pub extern "C" fn canvas_context_set_clear_color(context: *const RefCell<CanvasContext>, r: f32, g: f32, b: f32, a: f32) {
    let ctx = canvas_context_from_pointer(context);
    ctx.borrow_mut().set_clear_color(r, g, b, a);
}
pub extern "C" fn canvas_context_root(context: *const RefCell<CanvasContext>) -> *mut TreeNodeRc<Element> {
    let ctx = canvas_context_from_pointer(context);
    let mut ctx = ctx.borrow_mut();
    Box::into_raw(Box::new(ctx.root().clone()))
}
pub extern "C" fn release_node(node_pointer: *mut TreeNodeRc<Element>) {
    unsafe { Box::from_raw(node_pointer) };
}

#[inline]
fn node_from_pointer(node_pointer: *mut TreeNodeRc<Element>) -> TreeNodeRc<Element> {
    let node = unsafe { Box::from_raw(node_pointer) };
    let ret = (*node).clone();
    Box::into_raw(node);
    ret
}
pub enum ElementType {
    Empty = 0,
    Text = 1,
    Image = 2,
}
pub extern "C" fn element_new(context: *const RefCell<CanvasContext>, elem_type: ElementType) -> *mut TreeNodeRc<Element> {
    let ctx = canvas_context_from_pointer(context);
    let mut ctx = ctx.borrow_mut();
    let cfg = ctx.canvas_config();
    macro_rules! create_element {
        ($t: tt) => {
            {
                let mut temp_content = Box::new($t::new(&cfg));
                Box::new(TreeNodeRc::new(Element::new(&cfg, temp_content)))
            }
        }
    }
    let elem = match elem_type {
        ElementType::Empty => create_element!(Empty),
        ElementType::Text => create_element!(Text),
        ElementType::Image => create_element!(Image),
    };
    Box::into_raw(elem)
}
pub extern "C" fn element_parent() {}
pub extern "C" fn element_child() {}
pub extern "C" fn element_append() {}
pub extern "C" fn element_insert() {}
pub extern "C" fn element_remove() {}
pub extern "C" fn element_add_event_listener() {}
pub extern "C" fn element_remove_event_listener() {}
pub extern "C" fn element_dispatch_event() {}
pub extern "C" fn element_class() {}
pub extern "C" fn element_style() {}

pub extern "C" fn text_element_set_text() {}
pub extern "C" fn image_element_load() {}
