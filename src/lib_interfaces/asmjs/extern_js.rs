use std::ffi::CStr;
use std::os::raw::c_char;
use std::rc::Rc;
use std::cell::RefCell;
use super::super::super::tree::{TreeNodeRc, TreeNode};
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
pub extern "C" fn canvas_context_root(context: *const RefCell<CanvasContext>) -> *const TreeNode<Element> {
    let ctx = canvas_context_from_pointer(context);
    let mut ctx = ctx.borrow_mut();
    TreeNodeRc::into_ptr(ctx.root().clone())
}
#[no_mangle]
pub extern "C" fn release_node(node_pointer: *const TreeNode<Element>) {
    unsafe { TreeNodeRc::from_ptr(node_pointer, false) };
}

#[inline]
fn node_from_pointer(node_pointer: *const TreeNode<Element>) -> TreeNodeRc<Element> {
    let ret = unsafe { TreeNodeRc::from_ptr(node_pointer, true) };
    ret
}
pub enum ElementType {
    Empty = 0,
    Text = 1,
    Image = 2,
}
#[no_mangle]
pub extern "C" fn element_new(context: *const RefCell<CanvasContext>, elem_type: ElementType) -> *const TreeNode<Element> {
    let ctx = canvas_context_from_pointer(context);
    let mut ctx = ctx.borrow_mut();
    let cfg = ctx.canvas_config();
    macro_rules! create_element {
        ($t: tt) => {
            {
                let mut temp_content = Box::new($t::new(&cfg));
                TreeNodeRc::new(Element::new(&cfg, temp_content))
            }
        }
    }
    let elem = match elem_type {
        ElementType::Empty => create_element!(Empty),
        ElementType::Text => create_element!(Text),
        ElementType::Image => create_element!(Image),
    };
    TreeNodeRc::into_ptr(elem)
}
#[no_mangle]
pub extern "C" fn element_clone_node(node_pointer: *const TreeNode<Element>) -> *const TreeNode<Element> {
    let node = node_from_pointer(node_pointer);
    TreeNodeRc::into_ptr(node.clone_node())
}
#[no_mangle]
pub extern "C" fn element_parent(node_pointer: *const TreeNode<Element>) -> *const TreeNode<Element> {
    let node = node_from_pointer(node_pointer);
    let parent = node.parent();
    match parent {
        Some(p) => TreeNodeRc::into_ptr(p),
        None => 0 as *const TreeNode<Element>
    }
}
#[no_mangle]
pub extern "C" fn element_child(node_pointer: *const TreeNode<Element>, index: i32) -> *const TreeNode<Element> {
    let node = node_from_pointer(node_pointer);
    let index = index as usize;
    if index >= node.len() {
        0 as *const TreeNode<Element>
    } else {
        TreeNodeRc::into_ptr(node.child(index))
    }
}
#[no_mangle]
pub extern "C" fn element_append(node_pointer: *const TreeNode<Element>, child_node_pointer: *const TreeNode<Element>) {
    let mut node = node_from_pointer(node_pointer);
    let child = node_from_pointer(child_node_pointer);
    node.append(child);
}
#[no_mangle]
pub extern "C" fn element_insert(node_pointer: *const TreeNode<Element>, child_node_pointer: *const TreeNode<Element>, pos: i32) {
    let mut node = node_from_pointer(node_pointer);
    let child = node_from_pointer(child_node_pointer);
    let pos = pos as usize;
    node.insert(child, pos);
}
#[no_mangle]
pub extern "C" fn element_remove(node_pointer: *const TreeNode<Element>, pos: i32) {
    let mut node = node_from_pointer(node_pointer);
    node.remove(pos as usize);
}
#[no_mangle]
pub extern "C" fn element_replace(node_pointer: *const TreeNode<Element>, child_node_pointer: *const TreeNode<Element>, pos: i32) {
    let mut node = node_from_pointer(node_pointer);
    let child = node_from_pointer(child_node_pointer);
    let pos = pos as usize;
    node.replace(child, pos);
}
#[no_mangle]
pub extern "C" fn element_splice(node_pointer: *const TreeNode<Element>, pos: i32, length: i32, other_node_pointer: *const TreeNode<Element>) {
    let mut node = node_from_pointer(node_pointer);
    let pos = if pos < 0 { node.len() } else { pos as usize };
    if other_node_pointer == 0 as *const TreeNode<Element> {
        node.splice(pos, length as usize, None);
    } else {
        let child = node_from_pointer(other_node_pointer);
        node.splice(pos, length as usize, Some(child));
    }
}
#[no_mangle]
pub extern "C" fn element_find_child_position(node_pointer: *const TreeNode<Element>, child_node_pointer: *const TreeNode<Element>) -> i32 {
    let node = node_from_pointer(node_pointer);
    let child = node_from_pointer(child_node_pointer);
    match node.find_child_position(&child) {
        Some(x) => x as i32,
        None => -1
    }
}
#[no_mangle]
pub extern "C" fn element_length(node_pointer: *const TreeNode<Element>) -> i32 {
    let node = node_from_pointer(node_pointer);
    node.len() as i32
}
#[no_mangle]
pub extern "C" fn element_node_under_point(node_pointer: *const TreeNode<Element>, x: f64, y: f64) -> *const TreeNode<Element> {
    let node = node_from_pointer(node_pointer);
    let ret = node.elem().node_under_point(Point::new(x, y));
    match ret {
        Some(ret) => TreeNodeRc::into_ptr(ret),
        None => 0 as *const TreeNode<Element>
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
pub extern "C" fn element_tag_name(node_pointer: *const TreeNode<Element>, tag_name: *mut c_char) {
    let node = node_from_pointer(node_pointer);
    node.elem().tag_name(string_from_c_char_ptr(tag_name));
}
#[no_mangle]
pub extern "C" fn element_id(node_pointer: *const TreeNode<Element>, id: *mut c_char) {
    let node = node_from_pointer(node_pointer);
    node.elem().id(string_from_c_char_ptr(id));
}
#[no_mangle]
pub extern "C" fn element_class(node_pointer: *const TreeNode<Element>, class_names: *mut c_char) {
    let node = node_from_pointer(node_pointer);
    node.elem().class(string_from_c_char_ptr(class_names));
}
#[no_mangle]
pub extern "C" fn element_style(node_pointer: *const TreeNode<Element>, style_text: *mut c_char) {
    let node = node_from_pointer(node_pointer);
    node.elem().style_inline_text(str_from_c_char_ptr(style_text));
}
#[no_mangle]
pub extern "C" fn text_element_set_text(node_pointer: *const TreeNode<Element>, text: *mut c_char) {
    let node = node_from_pointer(node_pointer);
    node.elem().content_mut().downcast_mut::<Text>().unwrap().set_text(string_from_c_char_ptr(text));
}
#[no_mangle]
pub extern "C" fn image_element_load(node_pointer: *const TreeNode<Element>, url: *mut c_char) {
    let node = node_from_pointer(node_pointer);
    // FIXME image loader reuse
    node.elem().content_mut().downcast_mut::<Image>().unwrap().load(string_from_c_char_ptr(url));
}
// FIXME added image_element_get_natural_size interface
