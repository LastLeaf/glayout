#![macro_use]

use std::rc::Rc;
use std::cell::RefCell;
use super::utils::PretendSend;
use std::collections::HashMap;

pub type TestCaseFn = Fn() -> i32;

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
    pub fn run(name: &String) -> i32 {
        TEST_CASE_MAP.borrow().get(name).unwrap()()
    }
}

#[macro_export]
macro_rules! run_test_case {
    ($x:ident) => {
        $crate::test::TestManager::run(&$x)
    }
}

#[macro_export]
macro_rules! register_test_case {
    ($x:expr, $b:block) => {
        $crate::test::TestManager::register(String::from($x), Box::new(|| $b))
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
}
