use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::any::Any;
use super::Element;
use rc_forest::ForestNodeRc;

pub type EventCallback = Rc<RefCell<FnMut(&mut Element, &Event) + 'static>>;

pub struct Event<'a> {
    pub name: String,
    pub target: &'a ForestNodeRc<Element>,
    pub current_target: &'a ForestNodeRc<Element>,
    pub detail: &'a Box<Any + 'static>
}

impl<'a> Event<'a> {
    pub fn new(name: String, target: &'a ForestNodeRc<Element>, current_target: &'a ForestNodeRc<Element>, detail: &'a Box<Any + 'static>) -> Self {
        Self {
            name,
            target,
            current_target,
            detail,
        }
    }
    pub fn dispatch(self, element: &mut Element) {
        match element.event_receiver.get_listeners(&self.name) {
            None => { },
            Some(x) => {
                for listener in x.iter() {
                    let f = &mut *listener.borrow_mut();
                    f(element, &self);
                }
            }
        };
    }
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
    pub fn get_listeners(&self, name: &String) -> Option<Vec<EventCallback>> {
        self.listeners.get(name).map(|x| x.clone())
    }
}
