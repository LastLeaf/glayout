#![macro_use]

use super::utils::PretendSend;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type TestCaseFn = Fn() -> i32;

lazy_static! {
    static ref TEST_CASE_MAP: Arc<Mutex<PretendSend<HashMap<String, Box<TestCaseFn>>>>> = Arc::new(Mutex::new(PretendSend::new(HashMap::new())));
}

pub struct TestManager { }

impl TestManager {
    pub fn register(name: String, f: Box<TestCaseFn>) {
        TEST_CASE_MAP.lock().unwrap().insert(name, f);
    }
    pub fn run(name: &String) -> i32 {
        TEST_CASE_MAP.lock().unwrap().get(name).unwrap()()
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

pub fn init() {
    animation::init();
    canvas::init();
}
