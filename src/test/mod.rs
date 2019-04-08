#![macro_use]

use std::rc::Rc;
use std::cell::RefCell;
use super::utils::PretendSend;
use std::collections::BTreeMap;
use std::time::Duration;
use glayout::canvas::{Canvas, CanvasContext, TouchEventDetail};
use glayout::canvas::element::{Element, Empty, Text, Event};
use glayout::canvas::element::style::{DisplayType};

pub type TestCaseFn = Fn(Rc<RefCell<CanvasContext>>) -> i32;

lazy_static! {
    static ref TEST_CASE_MAP: PretendSend<Rc<RefCell<BTreeMap<String, Box<TestCaseFn>>>>> = PretendSend::new(Rc::new(RefCell::new(BTreeMap::new())));
    static ref MAIN_CANVAS_CONTEXT: PretendSend<RefCell<Option<Rc<RefCell<CanvasContext>>>>> = PretendSend::new(RefCell::new(None));
}

pub struct TestManager { }

impl TestManager {
    pub fn register(name: String, f: Box<TestCaseFn>) {
        let name = String::from(name.splitn(2, "test::").nth(1).unwrap());
        debug!("Registering test case: {}", name);
        TEST_CASE_MAP.borrow_mut().insert(name, f);
    }
    pub fn run(name: &String) -> i32 {
        debug!("Running test case: {}", name);
        TEST_CASE_MAP.borrow().get(name).unwrap()(MAIN_CANVAS_CONTEXT.borrow().as_ref().unwrap().clone())
    }
}

#[macro_export]
macro_rules! register_test_case {
    ($name:expr, $ctx:ident, $code:block) => {
        $crate::test::TestManager::register(String::from($name), Box::new(|$ctx| $code))
    }
}

#[macro_export]
macro_rules! run_test_case {
    ($name:expr) => {
        $crate::test::TestManager::run(&$name)
    }
}

// test cases

mod animation;
mod canvas;
mod element;
mod mouse_event;
mod painting;
mod layout;

pub fn init() {
    animation::init();
    canvas::init();
    element::init();
    mouse_event::init();
    painting::init();
    layout::init();

    let canvas = Canvas::new(0);
    *MAIN_CANVAS_CONTEXT.borrow_mut() = Some(canvas.context());
    show_list(canvas.context());
}

fn show_list(rc_context: Rc<RefCell<CanvasContext>>) {
    let rc_context_clone = rc_context.clone();
    let mut context = rc_context.borrow_mut();
    let pixel_ratio = context.device_pixel_ratio();
    context.set_canvas_size(800, 600, pixel_ratio);
    context.set_clear_color(0.5, 0.5, 0.5, 1.);

    let cfg = context.canvas_config();
    {
        let mut root = context.root().borrow_mut();
        cfg.append_style_sheet(&mut root, ".red { color: red; font-size: 36px }");
        let wrapper = element! (&mut root, &cfg, Empty {
            Empty {
                class: "red";
                display: DisplayType::Block;
                Text {
                    set_text("Test cases:");
                };
            };
            Empty {
                id: String::from("list");
            };
        });
        root.append(wrapper);
    }

    let list = context.root().borrow().node_by_id("list").unwrap();
    for (k, _) in TEST_CASE_MAP.borrow_mut().iter() {
        let mut root = context.root().borrow_mut();
        let rc_context_clone_item = rc_context_clone.clone();
        let element = element!(&mut root, &cfg, Empty {
            display: DisplayType::Block;
            Text {
                id: k.clone();
                set_text(k.as_str());
                @ "touchend" => move |element: &mut Element, event: &Event| {
                    let detail = event.detail.downcast_ref::<TouchEventDetail>().unwrap();
                    debug!("Touched at {:?}", (detail.client_x, detail.client_y));
                    let name: String = event.current_target.deref_with(element.node_mut()).style().get_id().to_string();
                    {
                        let mut context = rc_context_clone_item.borrow_mut();
                        context.root().deref_mut_with(element.node_mut()).remove(0);
                    }
                    glayout::set_timeout(move || {
                        run_test_case!(name);
                    }, Duration::new(0, 0));
                };
            };
        });
        list.deref_mut_with(&mut root).append(element);
    }
}
