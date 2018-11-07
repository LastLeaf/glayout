use cssparser::{Delimiter, ParserInput, Parser, ParseError};
use std::collections::HashMap;
use super::ElementClass;

struct SelectorFragment {
    id: String,
    classes: Vec<String>,
}

impl SelectorFragment {
    fn new() -> Self {
        Self {
            id: String::new(),
            classes: vec![],
        }
    }
}

struct Selector {
    fragments: Vec<SelectorFragment>
}

impl Selector {
    fn new() -> Self {
        Self {
            fragments: vec![]
        }
    }
    fn get_index_classes() -> Vec<String> {
        // TODO
    }
}

pub struct StyleSheet {
    class_name_map: HashMap<String, Vec<(Selector, ElementClass)>>
}

impl StyleSheet {
    pub fn parse_inline_style(c: &mut ElementClass, text: &str) {
        let mut input = ParserInput::new(text);
        let mut parser = Parser::new(&mut input);
        Self::parse_declarations(&mut parser, c);
    }
    pub fn new_from_css(text: &str) -> Self {
        let class_name_map = HashMap::new();
        let mut ret = Self {
            class_name_map
        };
        ret.parse_css_text(text);
        ret
    }
    fn parse_css_text(&mut self, text: &str) {
        // parse the css string
        let mut input = ParserInput::new(text);
        let mut parser = Parser::new(&mut input);
        while !parser.is_exhausted() {
            match Self::parse_rule_set(&mut parser) {
                Ok((selector, class)) => {
                    for name in selector.get_index_classes() {
                        let has = self.class_name_map.contains_key(&name);
                        match has {
                            true => {
                                self.class_name_map[&name].push(class);
                            },
                            false => {
                                self.class_name_map.insert(name, vec![class]);
                            }
                        }
                    }
                },
                _ => { }
            }

        }
    }

    fn parse_rule_set<'a>(parser: &mut Parser<'a, '_>) -> Result<(Selector, ElementClass), ParseError<'a, ()>> {
        let selector_res = parser.parse_until_before(Delimiter::CurlyBracketBlock, Self::parse_selector);
        let class = ElementClass::new();
        let declarations_res = parser.parse_nested_block(|parser| {
            Self::parse_declarations(parser, &mut class)
        });
        match selector_res {
            Ok(selector) => Ok((selector, class)),
            Err(e) => {
                warn!("CSS ParseError {:?}", e);
                Err(e)
            },
        }
    }
    fn parse_selector<'a>(parser: &mut Parser<'a, '_>) -> Result<Selector, ParseError<'a, ()>> {
        let selector = Selector::new();
        parser.parse_comma_separated(|parser| {
            let frag = SelectorFragment::new();
            // parse id
            parser.try(|input| {
                match parser.expect_delim('#') {
                    Ok(_) => {
                        match parser.expect_ident_cloned() {
                            Ok(id) => {
                                frag.id = String::from(id.as_ref());
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
            });
            // parse class
            loop {
                if parser.try(|input| {
                    match parser.expect_delim('.') {
                        Ok(_) => {
                            match parser.expect_ident_cloned() {
                                Ok(id) => {
                                    frag.classes.push(String::from(id.as_ref()));
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
            Ok(())
        });
        Ok(selector)
    }
    fn parse_declarations<'a>(parser: &mut Parser<'a, '_>, class: &mut ElementClass) -> Result<(String, ElementClass), ParseError<'a, ()>> {

    }

    pub fn query_declarations<'a>(&'a self, name: &String) -> Vec<&'a ElementClass> {
        match self.class_name_map.get(name) {
            None => vec![],
            Some(x) => {
                x.iter().map(|(_selector, class)| {
                    class
                }).collect()
            }
        }
    }
}

pub struct StyleSheetGroup {
    sheets: Vec<StyleSheet>
}

impl StyleSheetGroup {
    pub fn new() -> Self {
        Self {
            sheets: vec![]
        }
    }
    pub fn append_style_sheet(&mut self, sheet: StyleSheet) {
        self.sheets.push(sheet);
    }
    pub fn query_declarations<'a>(&'a self, name: &String) -> Vec<&'a ElementClass> {
        let mut ret = vec![];
        for sheet in self.sheets.iter() {
            ret.append(&mut sheet.query_declarations(name))
        }
        ret
    }
}
