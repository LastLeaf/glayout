use std::ffi::CStr;
use std::os::raw::c_char;
use std::rc::Rc;
use std::cell::RefCell;
use super::super::super::tree::{TreeNodeRc};
use super::super::super::canvas::{Canvas, CanvasContext};
use super::super::super::canvas::element::{Element, Empty, Text, Image};

// canvas
#[no_mangle]
pub extern "C" fn canvas_create(index: i32) -> *mut Canvas {
    let box_canvas = Box::new(Canvas::new(index));
    Box::into_raw(box_canvas)
}
#[no_mangle]
pub extern "C" fn canvas_destroy(canvas: *mut Canvas) {
    unsafe { Box::from_raw(canvas) };
}
#[no_mangle]
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
#[no_mangle]
pub extern "C" fn canvas_context_set_canvas_size(context: *const RefCell<CanvasContext>, w: i32, h: i32, pixel_ratio: f64) {
    let ctx = canvas_context_from_pointer(context);
    ctx.borrow_mut().set_canvas_size(w, h, pixel_ratio);
}
#[no_mangle]
pub extern "C" fn canvas_context_set_clear_color(context: *const RefCell<CanvasContext>, r: f32, g: f32, b: f32, a: f32) {
    let ctx = canvas_context_from_pointer(context);
    ctx.borrow_mut().set_clear_color(r, g, b, a);
}
#[no_mangle]
pub extern "C" fn canvas_context_append_style_sheet(context: *const RefCell<CanvasContext>, style_text: *mut c_char) {
    let ctx = canvas_context_from_pointer(context);
    let mut ctx = ctx.borrow_mut();
    ctx.canvas_config().append_style_sheet(str_from_c_char_ptr(style_text));
}
#[no_mangle]
pub extern "C" fn canvas_context_root(context: *const RefCell<CanvasContext>) -> *mut TreeNodeRc<Element> {
    let ctx = canvas_context_from_pointer(context);
    let mut ctx = ctx.borrow_mut();
    Box::into_raw(Box::new(ctx.root().clone()))
}
#[no_mangle]
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
#[no_mangle]
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
#[no_mangle]
pub extern "C" fn element_parent(node_pointer: *mut TreeNodeRc<Element>) -> *mut TreeNodeRc<Element> {
    let node = node_from_pointer(node_pointer);
    let parent = node.parent();
    match parent {
        Some(p) => Box::into_raw(Box::new(p)),
        None => 0 as *mut TreeNodeRc<Element>
    }
}
#[no_mangle]
pub extern "C" fn element_child(node_pointer: *mut TreeNodeRc<Element>, index: i32) -> *mut TreeNodeRc<Element> {
    let node = node_from_pointer(node_pointer);
    let index = index as usize;
    if index >= node.len() {
        0 as *mut TreeNodeRc<Element>
    } else {
        Box::into_raw(Box::new(node.child(index)))
    }
}
#[no_mangle]
pub extern "C" fn element_append(node_pointer: *mut TreeNodeRc<Element>, child_node_pointer: *mut TreeNodeRc<Element>) {
    let mut node = node_from_pointer(node_pointer);
    let child = node_from_pointer(child_node_pointer);
    node.append(child);
}
#[no_mangle]
pub extern "C" fn element_insert(node_pointer: *mut TreeNodeRc<Element>, child_node_pointer: *mut TreeNodeRc<Element>, pos: i32) {
    let mut node = node_from_pointer(node_pointer);
    let child = node_from_pointer(child_node_pointer);
    let pos = pos as usize;
    node.insert(child, pos);
}
#[no_mangle]
pub extern "C" fn element_remove(node_pointer: *mut TreeNodeRc<Element>, pos: i32) {
    let mut node = node_from_pointer(node_pointer);
    node.remove(pos as usize);
}
#[no_mangle]
pub extern "C" fn element_node_under_point(node_pointer: *mut TreeNodeRc<Element>, x: f64, y: f64) -> *mut TreeNodeRc<Element> {
    let node = node_from_pointer(node_pointer);
    let ret = node.elem().node_under_point((x, y));
    match ret {
        Some(ret) => Box::into_raw(Box::new(ret)),
        None => 0 as *mut TreeNodeRc<Element>
    }
}

#[inline]
fn str_from_c_char_ptr<'a>(ptr: *mut c_char) -> &'a str {
    let c_str: &CStr = unsafe { CStr::from_ptr(ptr) };
    c_str.to_str().unwrap()
}
#[inline]
fn string_from_c_char_ptr(ptr: *mut c_char) -> String {
    let c_str: &CStr = unsafe { CStr::from_ptr(ptr) };
    c_str.to_str().unwrap().to_owned()
}
#[no_mangle]
pub extern "C" fn element_class(node_pointer: *mut TreeNodeRc<Element>, class_names: *mut c_char) {
    let node = node_from_pointer(node_pointer);
    node.elem().class(str_from_c_char_ptr(class_names));
}
#[no_mangle]
pub extern "C" fn element_style(node_pointer: *mut TreeNodeRc<Element>, style_text: *mut c_char) {
    let node = node_from_pointer(node_pointer);
    node.elem().style_inline_text(str_from_c_char_ptr(style_text));
}
#[no_mangle]
pub extern "C" fn text_element_set_text(node_pointer: *mut TreeNodeRc<Element>, text: *mut c_char) {
    let node = node_from_pointer(node_pointer);
    node.elem().content_mut().downcast_mut::<Text>().unwrap().set_text(string_from_c_char_ptr(text));
}
#[no_mangle]
pub extern "C" fn image_element_load(node_pointer: *mut TreeNodeRc<Element>, url: *mut c_char) {
    let node = node_from_pointer(node_pointer);
    // FIXME image loader reuse
    node.elem().content_mut().downcast_mut::<Image>().unwrap().load(string_from_c_char_ptr(url));
}
