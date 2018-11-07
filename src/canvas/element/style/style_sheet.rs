use std::rc::Rc;
use std::any::Any;
use cssparser::{Delimiter, Token, ParserInput, Parser, ParseError};
use std::collections::HashMap;
use super::{ElementClass, StyleName, DisplayType, PositionType};

#[derive(Clone)]
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

#[derive(Clone)]
struct Selector {
    fragments: Vec<SelectorFragment>
}

impl Selector {
    fn new() -> Self {
        Self {
            fragments: vec![]
        }
    }
    fn get_index_classes(&self) -> Vec<String> {
        let mut ret = vec![];
        for frag in self.fragments.iter() {
            if frag.classes.len() > 0 {
                ret.push(frag.classes[0].clone())
            }
        }
        ret
    }
}

pub struct StyleSheet {
    class_name_map: HashMap<String, Vec<(Selector, Rc<ElementClass>)>>
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
                Ok(r) => {
                    let classes = r.0.get_index_classes();
                    for name in classes {
                        let has = self.class_name_map.contains_key(&name);
                        match has {
                            true => {
                                self.class_name_map.get_mut(&name).unwrap().push(r.clone());
                            },
                            false => {
                                self.class_name_map.insert(name, vec![r.clone()]);
                            }
                        }
                    }
                },
                _ => { }
            }
        }
    }

    fn parse_rule_set<'a>(parser: &mut Parser<'a, '_>) -> Result<(Selector, Rc<ElementClass>), ParseError<'a, ()>> {
        let selector_res = parser.parse_until_before(Delimiter::CurlyBracketBlock, Self::parse_selector);
        let mut class = Rc::new(ElementClass::new());
        if parser.expect_curly_bracket_block().is_ok() {
            parser.parse_nested_block(|parser| {
                Self::parse_declarations(parser, Rc::get_mut(&mut class).unwrap())
            });
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
            if parser.try(|parser| {
                match parser.expect_delim('#') {
                    Ok(_) => {
                        match parser.expect_ident() {
                            Ok(id) => {
                                frag.id = String::from(id.as_ref());
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
                // do nothing
            }
            // parse class
            loop {
                if parser.try(|parser| {
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
                let token = parser.next().unwrap().clone();
                warn!("CSS ParseError {:?}", parser.new_unexpected_token_error::<()>(token));
            }
            if has_limits {
                selector.fragments.push(frag);
            }
            Ok(())
        });
        Ok(selector)
    }
    fn parse_declarations<'a>(parser: &mut Parser<'a, '_>, class: &mut ElementClass) -> Result<(), ParseError<'a, ()>> {
        loop {
            let r = parser.expect_ident();
            match r {
                Ok(key) => {
                    match parser.expect_delim(':') {
                        Ok(_) => {
                            parser.parse_until_after::<_, _, ()>(Delimiter::Semicolon, |parser| {
                                // do different parsing for different keys
                                let (style_name, value_res) = match key.as_ref() {
                                    "display" => {
                                        const mapping: [(&'static str, DisplayType); 5] = [
                                            ("none", DisplayType::None),
                                            ("block", DisplayType::Block),
                                            ("inline", DisplayType::Inline),
                                            ("inline-block", DisplayType::InlineBlock),
                                            ("flex", DisplayType::Flex),
                                        ];
                                        (StyleName::display, Self::parse_enum(parser, &mapping))
                                    },
                                    "position" => {
                                        const mapping: [(&'static str, PositionType); 5] = [
                                            ("static", PositionType::Static),
                                            ("relative", PositionType::Relative),
                                            ("absolute", PositionType::Absolute),
                                            ("fixed", PositionType::Fixed),
                                            ("sticky", PositionType::Sticky),
                                        ];
                                        (StyleName::position, Self::parse_enum(parser, &mapping))
                                    },
                                    // "left" => style_name!(left, f64),
                                    // "top" => style_name!(top, f64),
                                    // "width" => style_name!(width, f64),
                                    // "height" => style_name!(height, f64),
                                    // "font-family" => style_name!(font_family, String),
                                    // "font-size" => style_name!(font_size, f32),
                                    // "line-height" => style_name!(line_height, f32),
                                    // "color" => style_name!(color, (f32, f32, f32, f32)),
                                    // "background-color" => style_name!(color, (f32, f32, f32, f32)),
                                    // "opacity" => style_name!(opacity, f32),
                                    // "transform" => style_name!(transform, super::Transform),
                                    _ => {
                                        (StyleName::glayout_unrecognized, Err(parser.new_custom_error(())))
                                    }
                                };
                                match value_res {
                                    Ok(v) => {
                                        class.add_rule(style_name, v);
                                    },
                                    Err(e) => {
                                        warn!("CSS ParseError {:?}", e);
                                    }
                                };
                                if !parser.is_exhausted() {
                                    let token = parser.next().unwrap().clone();
                                    warn!("CSS ParseError {:?}", parser.new_unexpected_token_error::<()>(token));
                                }
                                Ok(())
                            });
                        },
                        Err(e) => {
                            warn!("CSS ParseError {:?}", e);
                            loop {
                                match parser.next() {
                                    Ok(token) => {
                                        if *token == Token::Semicolon {
                                            break
                                        }
                                    },
                                    Err(_) => {
                                        break
                                    }
                                };
                            }
                        }
                    }
                },
                Err(_) => break
            }
        }
        Ok(())
    }
    #[inline]
    fn parse_enum<'a, T: Send + Sync + Sized>(parser: &mut Parser<'a, '_>, mapping: &'static [(&'static str, T)]) -> Result<Box<Any + Send>, ParseError<'a, ()>> {
        {
            let r = parser.expect_ident();
            if r.is_ok() {
                let value = r.unwrap();
                for (s, t) in mapping {
                    if value == s {
                        return Ok(Box::new(t.clone()));
                    }
                }
            }
        }
        Err(parser.new_custom_error(()))
    }

    pub fn query_declarations<'a>(&'a self, name: &String) -> Vec<Rc<ElementClass>> {
        match self.class_name_map.get(name) {
            None => vec![],
            Some(x) => {
                x.iter().map(|(_selector, class)| {
                    class.clone()
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
    pub fn query_declarations<'a>(&'a self, name: &String) -> Vec<Rc<ElementClass>> {
        let mut ret = vec![];
        for sheet in self.sheets.iter() {
            ret.append(&mut sheet.query_declarations(name))
        }
        ret
    }
}
