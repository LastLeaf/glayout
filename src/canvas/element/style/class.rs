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
    pub fn add_rule(&mut self, name: StyleName, value: Box<dyn Any + Send>) {
        self.rules.push((name, value))
    }
    pub fn replace_rule(&mut self, name: StyleName, value: Box<dyn Any + Send>) {
        let p = self.rules.iter().position(|x| x.0 == name);
        match p {
            Some(p) => {
                self.rules.remove(p);
            },
            None => { }
        }
        self.add_rule(name, value)
    }
    pub fn iter_rules(&self) -> Iter<(StyleName, Box<dyn Any + Send>)> {
        self.rules.iter()
    }
    pub fn from_style_text(&mut self, text: &str) {
        self.rules.truncate(0);
        super::StyleSheet::parse_inline_style(self, text);
    }
    pub fn apply_to_style(&self, style: &mut super::ElementStyle) {
        for (name, value) in self.rules.iter() {
            self.apply_rule(style, name, value);
        }
    }

    #[inline]
    fn apply_rule(&self, style: &mut super::ElementStyle, name: &StyleName, value: &Box<dyn Any + Send>) {
        super::apply_rule_from_class(style, name, value);
    }
}
