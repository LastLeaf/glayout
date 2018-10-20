#![macro_use]

use std::rc::Rc;
use std::cell::RefCell;
use super::utils::PretendSend;
use std::collections::HashMap;
use glayout::canvas::{Canvas, CanvasContext};
use glayout::canvas::element::{Element, Empty, Image, Text};
use glayout::canvas::element::style::{DisplayType, PositionType};

pub type TestCaseFn = Fn(Rc<RefCell<CanvasContext>>) -> i32;

lazy_static! {
    static ref TEST_CASE_MAP: PretendSend<Rc<RefCell<HashMap<String, Box<TestCaseFn>>>>> = PretendSend::new(Rc::new(RefCell::new(HashMap::new())));
}

pub struct TestManager { }

impl TestManager {
    pub fn register(name: String, f: Box<TestCaseFn>) {
        let name = String::from(name.splitn(2, "test::").nth(1).unwrap());
        debug!("Registering test case: {}", name);
        TEST_CASE_MAP.borrow_mut().insert(name, f);
    }
    pub fn run(name: &String, ctx: Rc<RefCell<CanvasContext>>) -> i32 {
        TEST_CASE_MAP.borrow().get(name).unwrap()(ctx)
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
        unimplemented!();
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
    let name = show_list(canvas.context());
}

fn show_list(rc_context: Rc<RefCell<CanvasContext>>) {
    {
        let mut context = rc_context.borrow_mut();
        let mut root = context.root();
        let cfg = context.canvas_config();
        let wrapper = element! (&cfg, Empty {
            Empty {
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
        let mut list = context.node_by_id("list").unwrap();
        for (k, _) in TEST_CASE_MAP.borrow_mut().iter() {
            list.append(element! (&cfg, Empty {
                display: DisplayType::Block;
                Text {
                    set_text(k.as_str());
                };
            }));
        }
    }
}

fn run_from_list() {

}
