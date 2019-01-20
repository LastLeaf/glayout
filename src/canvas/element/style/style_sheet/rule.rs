use std::rc::Rc;
use super::{ElementClass};
use super::selector::Selector;

#[derive(Clone)]
pub(super) struct Rule {
    pub(super) selector: Selector,
    pub(super) rc_class: Rc<ElementClass>,
    pub(super) priority: i32,
}

impl Rule {
    pub(super) fn new(selector: Selector, rc_class: Rc<ElementClass>, priority: i32) -> Self {
        Self {
            selector,
            rc_class,
            priority,
        }
    }
}
