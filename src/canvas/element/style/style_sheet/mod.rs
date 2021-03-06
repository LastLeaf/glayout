use std::rc::Rc;
use cssparser::{Delimiter, Token, ParserInput, Parser, ParseError, Color};
use std::collections::HashMap;
pub(self) use super::*;

mod selector;
use self::selector::{Selector, SelectorFragment, SelectorQuery};
mod rule;
use self::rule::Rule;
mod value_parser;
use self::value_parser::parse_value;

pub(crate) struct StyleSheet {
    unindexed_classes: Vec<Rc<Rule>>,
    class_name_map: HashMap<String, Vec<Rc<Rule>>>,
}

impl StyleSheet {
    pub fn parse_inline_style(c: &mut ElementClass, text: &str) {
        let mut input = ParserInput::new(text);
        let mut parser = Parser::new(&mut input);
        Self::parse_declarations(&mut parser, c).unwrap();
    }
    pub fn new_from_css(text: &str) -> Self {
        let class_name_map = HashMap::new();
        let mut ret = Self {
            unindexed_classes: vec![],
            class_name_map
        };
        ret.parse_css_text(text);
        ret
    }
    fn parse_css_text(&mut self, text: &str) {
        // parse the css string
        let mut input = ParserInput::new(text);
        let mut parser = Parser::new(&mut input);
        let mut rule_index = 0;
        while !parser.is_exhausted() {
            match Self::parse_rule_set(&mut parser) {
                Ok(r) => {
                    let rule = Rc::new(Rule::new(r.0.clone(), r.1, rule_index));
                    let classes = r.0.get_index_classes();
                    for name in classes {
                        if name == "" {
                            self.unindexed_classes.push(rule.clone());
                        } else {
                            let has = self.class_name_map.contains_key(&name);
                            match has {
                                true => {
                                    self.class_name_map.get_mut(&name).unwrap().push(rule.clone());
                                },
                                false => {
                                    self.class_name_map.insert(name, vec![rule.clone()]);
                                }
                            }
                        }
                    }
                },
                _ => { }
            }
            rule_index += 1;
        }
    }

    fn parse_rule_set<'a>(parser: &mut Parser<'a, '_>) -> Result<(Selector, Rc<ElementClass>), ParseError<'a, ()>> {
        let selector_res = parser.parse_until_before(Delimiter::CurlyBracketBlock, Self::parse_selector);
        let mut class = Rc::new(ElementClass::new());
        if parser.expect_curly_bracket_block().is_ok() {
            parser.parse_nested_block(|parser| {
                Self::parse_declarations(parser, Rc::get_mut(&mut class).unwrap())
            }).unwrap();
        }
        match selector_res {
            Ok(selector) => Ok((selector, class)),
            Err(e) => {
                warn!("CSS ParseError {:?}", e);
                Err(e)
            },
        }
    }
    fn parse_selector<'a>(parser: &mut Parser<'a, '_>) -> Result<Selector, ParseError<'a, ()>> {
        let mut selector = Selector::new();
        parser.parse_comma_separated::<_, _, ()>(|parser| {
            let mut frag = SelectorFragment::new();
            let mut has_limits = false;
            // parse id
            if parser.r#try(|parser| {
                match parser.expect_ident() {
                    Ok(tag_name) => {
                        frag.tag_name = String::from(tag_name.as_ref());
                        has_limits = true;
                        Ok(())
                    },
                    Err(e) => Err(e)
                }
            }).is_err() {
                // do nothing
            }
            // parse id
            if parser.r#try(|parser| {
                match parser.next() {
                    Ok(token) => {
                        match token {
                            Token::IDHash(id) => {
                                frag.id = String::from(id.as_ref());
                                has_limits = true;
                                Ok(())
                            },
                            _ => Err(())
                        }
                    },
                    Err(_) => Err(())
                }
            }).is_err() {
                // do nothing
            }
            // parse class
            while !parser.is_exhausted() {
                if parser.r#try(|parser| {
                    match parser.expect_delim('.') {
                        Ok(_) => {
                            match parser.expect_ident() {
                                Ok(id) => {
                                    frag.classes.push(String::from(id.as_ref()));
                                    has_limits = true;
                                    Ok(())
                                },
                                Err(e) => {
                                    warn!("CSS ParseError {:?}", e);
                                    Err(e)
                                }
                            }
                        },
                        Err(e) => Err(e)
                    }
                }).is_err() {
                    break;
                }
            }
            if !parser.is_exhausted() {
                {
                    if parser.next().is_err() {
                        // do nothing
                    }
                }
                warn!("CSS ParseError {:?}", parser.new_custom_error::<_, ()>(()));
            }
            while !parser.is_exhausted() {
                if parser.next().is_err() {
                    // do nothing
                }
            }
            if has_limits {
                selector.fragments.push(frag);
            }
            Ok(())
        }).unwrap();
        Ok(selector)
    }
    fn parse_declarations<'a>(parser: &mut Parser<'a, '_>, class: &mut ElementClass) -> Result<(), ParseError<'a, ()>> {
        while !parser.is_exhausted() {
            let key = {
                let r = parser.expect_ident();
                if r.is_err() {
                    warn!("CSS ParseError {:?}", r.unwrap_err());
                    continue;
                }
                String::from(r.unwrap().as_ref())
            };
            while !parser.is_exhausted() {
                let r = parser.expect_colon();
                if r.is_err() {
                    warn!("CSS ParseError {:?}", r.unwrap_err());
                } else {
                    break;
                }
            };
            parser.parse_until_after::<_, _, ()>(Delimiter::Semicolon, |parser| {
                // match parser.r#try(|parser| {
                //     let r = parser.expect_ident();
                //     match r {
                //         Err(_) => Err(parser.new_custom_error::<_, ()>(())),
                //         Ok(r) => {
                //             if r.as_ref() == "inherit" {
                //                 unimplemented!("Style value \"inherit\" is not supported yet");
                //             } else {
                //                 Err(parser.new_custom_error::<_, ()>(()))
                //             }
                //         }
                //     }
                // }) {
                //     Ok(x) => Ok(x),
                //     Err(_) => {
                        parse_value(parser, class, key.as_ref());
                        if !parser.is_exhausted() {
                            {
                                if parser.next().is_err() {
                                    // do nothing
                                }
                            }
                            warn!("CSS ParseError {:?}", parser.new_custom_error::<_, ()>(()));
                        }
                        while !parser.is_exhausted() {
                            if parser.next().is_err() {
                                // do nothing
                            }
                        }
                        Ok(())
                //     }
                // }
            }).unwrap();
        }
        Ok(())
    }
    fn query_declarations<'a>(&'a self, cond: &SelectorQuery) -> Vec<Rc<ElementClass>> {
        let mut ret: Vec<Rc<Rule>> = vec![];
        {
            let mut match_and_insert = |rule: &Rc<Rule>| {
                if rule.selector.match_query(cond) {
                    let index = ret.iter().rposition(|r| {
                        r.priority <= rule.priority
                    });
                    match index {
                        Some(index) => {
                            if !Rc::ptr_eq(&ret[index], rule) {
                                ret.insert(index + 1, rule.clone());
                            }
                        },
                        None => {
                            ret.insert(0, rule.clone());
                        }
                    }
                }
            };
            for rule in self.unindexed_classes.iter() {
                match_and_insert(rule);
            }
            for s in cond.classes.iter() {
                match self.class_name_map.get(&String::from(*s)) {
                    None => {},
                    Some(rules) => {
                        for rule in rules {
                            match_and_insert(rule);
                        }
                    }
                }
            }
        }
        ret.into_iter().map(|x| x.rc_class.clone()).collect()
    }
}

pub(crate) struct StyleSheetGroup {
    sheets: Vec<StyleSheet>
}

impl StyleSheetGroup {
    pub fn new() -> Self {
        Self {
            sheets: vec![]
        }
    }
    pub fn len(&self) -> usize {
        self.sheets.len()
    }
    pub fn append(&mut self, sheet: StyleSheet) {
        self.sheets.push(sheet);
    }
    pub fn replace(&mut self, index: usize, sheet: StyleSheet) {
        self.sheets[index] = sheet;
    }
    pub fn clear(&mut self) {
        self.sheets.truncate(0);
    }
    pub fn query_declarations<'a>(&'a self, tag_name: &'a str, id: &'a str, classes: Box<[&'a str]>) -> Vec<Rc<ElementClass>> {
        let sq = SelectorQuery::new(tag_name, id, classes);
        let mut ret = vec![];
        for sheet in self.sheets.iter() {
            ret.append(&mut sheet.query_declarations(&sq))
        }
        ret
    }
}

#[cfg(test)]
mod test {
    use super::{StyleSheet, StyleSheetGroup, StyleName};

    #[test]
    fn query_declarations() {
        let mut ssg = StyleSheetGroup::new();
        let ss = StyleSheet::new_from_css("
            .a {
                position: absolute;
                display: block;
            }
            .b { display: none }
            .a { display: none }
        ");
        ssg.append(ss);
        let ss = StyleSheet::new_from_css("
            .a { left: 0 }
        ");
        ssg.append(ss);
        let classes = ssg.query_declarations("", "", Box::new(["a"]));
        assert_eq!(classes.len(), 3);
        assert_eq!(classes[0]._iter_rules().next().unwrap().0, StyleName::position);
        assert_eq!(classes[1]._iter_rules().next().unwrap().0, StyleName::display);
        assert_eq!(classes[2]._iter_rules().next().unwrap().0, StyleName::left);
    }
}
