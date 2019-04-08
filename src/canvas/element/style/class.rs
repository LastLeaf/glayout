use std::any::Any;
use std::slice::Iter;
use super::StyleName;

#[derive(Default)]
pub struct ElementClass {
    rules: Vec<(StyleName, Box<dyn Any + Send>)>,
}

impl ElementClass {
    pub fn new() -> Self {
        Self {
            rules: vec![]
        }
    }
    pub(super) fn add_rule(&mut self, name: StyleName, value: Box<dyn Any + Send>) {
        self.rules.push((name, value))
    }
    pub(super) fn replace_rule(&mut self, name: StyleName, value: Box<dyn Any + Send>) {
        let p = self.rules.iter().position(|x| x.0 == name);
        match p {
            Some(p) => {
                self.rules.remove(p);
            },
            None => { }
        }
        self.add_rule(name, value)
    }
    pub(super) fn _iter_rules(&self) -> Iter<(StyleName, Box<dyn Any + Send>)> {
        self.rules.iter()
    }
    pub fn apply_to_style(&self, style: &super::ElementStyle) {
        for (name, value) in self.rules.iter() {
            super::apply_rule_from_class(style, name, value);
        }
    }
}
