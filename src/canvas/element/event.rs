use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::any::Any;
use super::Element;
use super::super::super::tree::{TreeNodeRc};

pub type EventCallback = Rc<RefCell<FnMut(&Event) + 'static>>;

pub struct Event<'a> {
    pub name: String,
    pub target: TreeNodeRc<Element>,
    pub current_target: TreeNodeRc<Element>,
    pub detail: &'a Box<Any + 'static>
}

pub struct EventReceiver {
    listeners: HashMap<String, Vec<EventCallback>>
}

impl EventReceiver {
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new()
        }
    }
    pub fn has_listener(&mut self, name: String) -> bool {
        match self.listeners.get(&name) {
            None => { false },
            Some(x) => {
                match x.len() {
                    0 => { false },
                    _ => { true }
                }
            }
        }
    }
    pub fn add_listener(&mut self, name: String, f: EventCallback) {
        if self.listeners.contains_key(&name) {
            self.listeners.get_mut(&name).unwrap().push(f);
        } else {
            self.listeners.insert(name, vec![f]);
        };
    }
    pub fn remove_listener(&mut self, name: String, f: EventCallback) -> bool {
        match self.listeners.get_mut(&name) {
            None => { false },
            Some(x) => {
                let index = x.iter().position(|x| Rc::ptr_eq(x, &f));
                match index {
                    None => { false },
                    Some(index) => {
                        x.remove(index);
                        true
                    }
                }
            }
        }
    }
    fn dispatch_event<'a, 'b>(&self, event: &'b Event<'a>) {
        let name = event.name.clone();
        match self.listeners.get(&name) {
            None => { },
            Some(x) => {
                for listener in x.iter() {
                    let f = &mut *listener.borrow_mut();
                    f(&event);
                }
            }
        };
    }
    pub fn new_event<'a>(&self, name: String, target: TreeNodeRc<Element>, current_target: TreeNodeRc<Element>, detail: &'a Box<Any + 'static>) {
        self.dispatch_event(&Event {
            name,
            target,
            current_target,
            detail,
        });
    }
}
