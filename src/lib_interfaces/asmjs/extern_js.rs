use std::ffi::CStr;
use std::os::raw::c_char;
use std::rc::Rc;
use std::cell::RefCell;
use rc_forest::{ForestNodeRc, ForestNode, ForestNodePtr};
use super::super::super::canvas::{Canvas, CanvasContext};
use super::super::super::canvas::element::{Element, Empty, Text, Image, Point};

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
pub extern "C" fn canvas_context_clear_style_sheets(context: *const RefCell<CanvasContext>) {
    let ctx = canvas_context_from_pointer(context);
    let mut ctx = ctx.borrow_mut();
    ctx.canvas_config().clear_style_sheets();
}
#[no_mangle]
pub extern "C" fn canvas_context_root(context: *const RefCell<CanvasContext>) -> ForestNodePtr<Element> {
    let ctx = canvas_context_from_pointer(context);
    let mut ctx = ctx.borrow_mut();
    ForestNodeRc::into_ptr(ctx.root().clone())
}
#[no_mangle]
pub extern "C" fn release_node(node_pointer: ForestNodePtr<Element>) {
    unsafe { ForestNodeRc::from_ptr(node_pointer, false) };
}

#[inline]
fn node_rc_from_pointer(node_pointer: ForestNodePtr<Element>) -> ForestNodeRc<Element> {
    unsafe { ForestNodeRc::from_ptr(node_pointer, true) }
}
#[inline]
fn node_from_pointer<'a>(node_pointer: ForestNodePtr<Element>) -> &'a mut ForestNode<Element> {
    unsafe { ForestNodeRc::from_ptr(node_pointer, true).forest_node_mut() }
}
pub enum ElementType {
    Empty = 0,
    Text = 1,
    Image = 2,
}
#[no_mangle]
pub extern "C" fn element_new(context: *const RefCell<CanvasContext>, elem_type: ElementType) -> ForestNodePtr<Element> {
    let ctx = canvas_context_from_pointer(context);
    let mut ctx = ctx.borrow_mut();
    let cfg = ctx.canvas_config();
    let root = unsafe { ctx.root().forest_node_mut() };
    macro_rules! create_element {
        ($t: tt) => {
            {
                let mut temp_content = Box::new($t::new(&cfg));
                root.create_another(Element::new(&cfg, temp_content))
            }
        }
    }
    let elem = match elem_type {
        ElementType::Empty => create_element!(Empty),
        ElementType::Text => create_element!(Text),
        ElementType::Image => create_element!(Image),
    };
    ForestNodeRc::into_ptr(elem)
}
#[no_mangle]
pub extern "C" fn element_clone_node(node_pointer: ForestNodePtr<Element>) -> ForestNodePtr<Element> {
    let node = node_rc_from_pointer(node_pointer);
    ForestNodeRc::into_ptr(node.clone_node_with(unsafe {node.forest_node_mut()}))
}
#[no_mangle]
pub extern "C" fn element_parent(node_pointer: ForestNodePtr<Element>) -> ForestNodePtr<Element> {
    let node = node_from_pointer(node_pointer);
    let parent = node.parent();
    match parent {
        Some(p) => ForestNodeRc::into_ptr(p.rc()),
        None => 0 as ForestNodePtr<Element>
    }
}
#[no_mangle]
pub extern "C" fn element_child(node_pointer: ForestNodePtr<Element>, index: i32) -> ForestNodePtr<Element> {
    let node = node_from_pointer(node_pointer);
    let index = index as usize;
    match node.child(index) {
        None => 0 as ForestNodePtr<Element>,
        Some(node) => ForestNodeRc::into_ptr(node.rc())
    }
}
#[no_mangle]
pub extern "C" fn element_append(node_pointer: ForestNodePtr<Element>, child_node_pointer: ForestNodePtr<Element>) {
    let node = node_from_pointer(node_pointer);
    let child = node_rc_from_pointer(child_node_pointer);
    node.append(child);
}
#[no_mangle]
pub extern "C" fn element_insert(node_pointer: ForestNodePtr<Element>, child_node_pointer: ForestNodePtr<Element>, pos: i32) {
    let node = node_from_pointer(node_pointer);
    let child = node_rc_from_pointer(child_node_pointer);
    let pos = pos as usize;
    node.insert(child, pos);
}
#[no_mangle]
pub extern "C" fn element_remove(node_pointer: ForestNodePtr<Element>, pos: i32) {
    let node = node_from_pointer(node_pointer);
    node.remove(pos as usize);
}
#[no_mangle]
pub extern "C" fn element_replace(node_pointer: ForestNodePtr<Element>, child_node_pointer: ForestNodePtr<Element>, pos: i32) {
    let node = node_from_pointer(node_pointer);
    let child = node_rc_from_pointer(child_node_pointer);
    let pos = pos as usize;
    node.replace(child, pos);
}
#[no_mangle]
pub extern "C" fn element_splice(node_pointer: ForestNodePtr<Element>, pos: i32, length: i32, other_node_pointer: ForestNodePtr<Element>) {
    let node = node_from_pointer(node_pointer);
    let pos = if pos < 0 { node.len() } else { pos as usize };
    if other_node_pointer == 0 as ForestNodePtr<Element> {
        node.splice(pos, length as usize, vec![]);
    } else {
        let other = node_rc_from_pointer(other_node_pointer);
        let children = other.deref_with(node).clone_children();
        node.splice(pos, length as usize, children);
    }
}
#[no_mangle]
pub extern "C" fn element_find_child_position(node_pointer: ForestNodePtr<Element>, child_node_pointer: ForestNodePtr<Element>) -> i32 {
    let node = node_from_pointer(node_pointer);
    let child = node_rc_from_pointer(child_node_pointer);
    match node.find_child_position(&child) {
        Some(x) => x as i32,
        None => -1
    }
}
#[no_mangle]
pub extern "C" fn element_length(node_pointer: ForestNodePtr<Element>) -> i32 {
    let node = node_from_pointer(node_pointer);
    node.len() as i32
}
#[no_mangle]
pub extern "C" fn element_node_under_point(node_pointer: ForestNodePtr<Element>, x: f64, y: f64) -> ForestNodePtr<Element> {
    let node = node_from_pointer(node_pointer);
    let ret = node.node_under_point(Point::new(x, y));
    match ret {
        Some(ret) => ForestNodeRc::into_ptr(ret),
        None => 0 as ForestNodePtr<Element>
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
pub extern "C" fn element_tag_name(node_pointer: ForestNodePtr<Element>, tag_name: *mut c_char) {
    let node = node_from_pointer(node_pointer);
    node.tag_name(string_from_c_char_ptr(tag_name));
}
#[no_mangle]
pub extern "C" fn element_id(node_pointer: ForestNodePtr<Element>, id: *mut c_char) {
    let node = node_from_pointer(node_pointer);
    node.id(string_from_c_char_ptr(id));
}
#[no_mangle]
pub extern "C" fn element_class(node_pointer: ForestNodePtr<Element>, class_names: *mut c_char) {
    let node = node_from_pointer(node_pointer);
    node.class(string_from_c_char_ptr(class_names));
}
#[no_mangle]
pub extern "C" fn element_style(node_pointer: ForestNodePtr<Element>, style_text: *mut c_char) {
    let node = node_from_pointer(node_pointer);
    node.style_inline_text(str_from_c_char_ptr(style_text));
}
#[no_mangle]
pub extern "C" fn text_element_set_text(node_pointer: ForestNodePtr<Element>, text: *mut c_char) {
    let node = node_from_pointer(node_pointer);
    node.content_mut().downcast_mut::<Text>().unwrap().set_text(string_from_c_char_ptr(text));
}
#[no_mangle]
pub extern "C" fn image_element_load(node_pointer: ForestNodePtr<Element>, url: *mut c_char) {
    let node = node_from_pointer(node_pointer);
    // FIXME image loader reuse
    node.content_mut().downcast_mut::<Image>().unwrap().load(string_from_c_char_ptr(url));
}
// FIXME added image_element_get_natural_size interface
