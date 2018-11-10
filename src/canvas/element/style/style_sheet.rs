use std::rc::Rc;
use std::any::Any;
use cssparser::{Delimiter, Token, ParserInput, Parser, ParseError, Color};
use std::collections::HashMap;
pub(self) use super::{ElementClass, StyleName, DisplayType, PositionType};

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
        Self::parse_declarations(&mut parser, c).unwrap();
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
            while !parser.is_exhausted() {
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
                macro_rules! add_rule {
                    ($style_name: expr, $value_res: expr) => {
                        match $value_res {
                            Ok(v) => {
                                class.add_rule($style_name, v);
                            },
                            Err(e) => {
                                warn!("CSS ParseError {:?}", e);
                            }
                        };
                    }
                }
                // do different parsing for different keys
                match key.as_ref() {
                    "display" => {
                        const MAPPING: [(&'static str, DisplayType); 5] = [
                            ("none", DisplayType::None),
                            ("block", DisplayType::Block),
                            ("inline", DisplayType::Inline),
                            ("inline-block", DisplayType::InlineBlock),
                            ("flex", DisplayType::Flex),
                        ];
                        add_rule!(StyleName::display, Self::parse_enum(parser, &MAPPING));
                    },
                    "position" => {
                        const MAPPING: [(&'static str, PositionType); 5] = [
                            ("static", PositionType::Static),
                            ("relative", PositionType::Relative),
                            ("absolute", PositionType::Absolute),
                            ("fixed", PositionType::Fixed),
                            ("sticky", PositionType::Sticky),
                        ];
                        add_rule!(StyleName::position, Self::parse_enum(parser, &MAPPING));
                    },
                    "left" => {
                        add_rule!(StyleName::left, Self::parse_length::<f64>(parser));
                    },
                    "top" => {
                        add_rule!(StyleName::top, Self::parse_length::<f64>(parser));
                    },
                    "width" => {
                        add_rule!(StyleName::width, Self::parse_length::<f64>(parser));
                    },
                    "height" => {
                        add_rule!(StyleName::height, Self::parse_length::<f64>(parser));
                    },
                    "font-family" => {
                        add_rule!(StyleName::font_family, Self::parse_font_family(parser));
                    },
                    "font-size" => {
                        add_rule!(StyleName::font_size, Self::parse_length::<f32>(parser));
                    },
                    "line-height" => {
                        add_rule!(StyleName::line_height, Self::parse_length::<f32>(parser));
                    },
                    "color" => {
                        add_rule!(StyleName::color, Self::parse_color(parser));
                    },
                    "background-color" => {
                        add_rule!(StyleName::background_color, Self::parse_color(parser));
                    },
                    "opacity" => {
                        add_rule!(StyleName::opacity, Self::parse_number::<f32>(parser));
                    },
                    "transform" => {
                        unimplemented!();
                    },
                    _ => {
                        add_rule!(StyleName::glayout_unrecognized, Err(parser.new_custom_error::<_, ()>(())));
                    }
                };
                if !parser.is_exhausted() {
                    let token = parser.next().unwrap().clone();
                    warn!("CSS ParseError {:?}", parser.new_unexpected_token_error::<()>(token));
                }
                Ok(())
            }).unwrap();
        }
        Ok(())
    }
    #[inline]
    fn parse_number<'a, T: 'static + From<f32> + Send + Sync + Clone>(parser: &mut Parser<'a, '_>) -> Result<Box<Any + Send>, ParseError<'a, ()>> {
        {
            let r = parser.next();
            if r.is_ok() {
                let token = r.unwrap();
                match token {
                    Token::Number {value, has_sign: _, int_value: _} => {
                        let num: f32 = *value;
                        let num_t: T = num.into();
                        return Ok(Box::new(num_t.clone()));
                    },
                    _ => { }
                }
            }
        }
        Err(parser.new_custom_error(()))
    }
    #[inline]
    fn parse_length<'a, T: 'static + From<f32> + Send + Sync + Clone>(parser: &mut Parser<'a, '_>) -> Result<Box<Any + Send>, ParseError<'a, ()>> {
        {
            let r = parser.next();
            if r.is_ok() {
                let token = r.unwrap();
                match token {
                    Token::Number {value, has_sign: _, int_value: _} => {
                        let num: f32 = *value;
                        let num_t: T = num.into();
                        return Ok(Box::new(num_t.clone()));
                    },
                    Token::Dimension {value, unit, has_sign: _, int_value: _} => {
                        let num: f32 = *value;
                        let num_t: T = num.into();
                        if unit.as_ref() == "px" {
                            return Ok(Box::new(num_t.clone()));
                        }
                    },
                    _ => { }
                }
            }
        }
        Err(parser.new_custom_error(()))
    }
    #[inline]
    fn parse_color<'a>(parser: &mut Parser<'a, '_>) -> Result<Box<Any + Send>, ParseError<'a, ()>> {
        match Color::parse(parser) {
            Ok(c) => {
                match c {
                    Color::RGBA(c) => {
                        Ok(Box::new((c.red_f32(), c.green_f32(), c.blue_f32(), c.alpha_f32())))
                    },
                    _ => Err(parser.new_custom_error(()))
                }
            },
            Err(_) => Err(parser.new_custom_error(()))
        }
    }
    #[inline]
    fn _parse_string_or_ident<'a>(parser: &mut Parser<'a, '_>) -> Result<Box<Any + Send>, ParseError<'a, ()>> {
        {
            let r = parser.next();
            if r.is_ok() {
                let token = r.unwrap();
                match token {
                    Token::QuotedString(s) => {
                        return Ok(Box::new(String::from(s.as_ref())));
                    },
                    Token::Ident(s) => {
                        return Ok(Box::new(String::from(s.as_ref())));
                    },
                    _ => { }
                }
            }
        }
        Err(parser.new_custom_error(()))
    }
    #[inline]
    fn parse_enum<'a, T: Clone + Send + Sync + Sized>(parser: &mut Parser<'a, '_>, mapping: &'static [(&'static str, T)]) -> Result<Box<Any + Send>, ParseError<'a, ()>> {
        {
            let r = parser.expect_ident();
            if r.is_ok() {
                let value = r.unwrap();
                for (s, t) in mapping {
                    if value == s {
                        let t: T = (*t).clone();
                        return Ok(Box::new(t));
                    }
                }
            }
        }
        Err(parser.new_custom_error(()))
    }
    #[inline]
    fn parse_font_family<'a>(parser: &mut Parser<'a, '_>) -> Result<Box<Any + Send>, ParseError<'a, ()>> {
        {
            let mut ret = vec![];
            let r = parser.parse_comma_separated(|parser| {
                {
                    let mut f = || {
                        {
                            let token = {
                                let r = parser.next();
                                if r.is_err() {
                                    return Err(());
                                }
                                r.unwrap()
                            };
                            match token {
                                Token::QuotedString(s) => {
                                    ret.push(String::new() + "\"" + s + "\"");
                                },
                                Token::Ident(s) => {
                                    ret.push(String::new() + "\"" + s + "\"");
                                },
                                _ => {
                                    return Err(());
                                }
                            }
                        }
                        if !parser.is_exhausted() {
                            return Err(());
                        }
                        Ok(())
                    };
                    f()
                }.map_err(|_| {
                    parser.new_custom_error(())
                })
            });
            if r.is_ok() {
                Ok(Box::new(ret.join(",")))
            } else {
                Err(r.unwrap_err())
            }
        }
    }

    pub fn query_declarations<'a>(&'a self, name: &str) -> Vec<Rc<ElementClass>> {
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
    pub fn append(&mut self, sheet: StyleSheet) {
        self.sheets.push(sheet);
    }
    pub fn query_declarations<'a>(&'a self, name: &str) -> Vec<Rc<ElementClass>> {
        let mut ret = vec![];
        for sheet in self.sheets.iter() {
            ret.append(&mut sheet.query_declarations(name))
        }
        ret
    }
}

#[cfg(test)]
mod test {
    use std::any::Any;
    use super::{StyleSheet, StyleSheetGroup, StyleName};

    #[test]
    fn parse_css_text() {
        let ss = StyleSheet::new_from_css("
            .a {
                display: block;
                position: absolute;
                left: 1px;
                top: 2.3px;
                width: 4px;
                height: 5px;
                font-family: \"宋体\", sans;
                font-size: 6px;
                line-height: 7px;
                color: red;
                background-color: #00ff00;
                opacity: 0.8;
            }
        ");
        let classes = ss.query_declarations("a");
        let rules: Vec<&(StyleName, Box<Any + Send>)> = classes[0].iter_rules().collect();
        assert_eq!(rules[0].0, StyleName::display);
        assert_eq!(rules[0].1.downcast_ref::<super::DisplayType>().unwrap().clone(), super::DisplayType::Block);
        assert_eq!(rules[1].0, StyleName::position);
        assert_eq!(rules[1].1.downcast_ref::<super::PositionType>().unwrap().clone(), super::PositionType::Absolute);
        assert_eq!(rules[2].0, StyleName::left);
        assert_eq!(*rules[2].1.downcast_ref::<f64>().unwrap(), 1. as f32 as f64);
        assert_eq!(rules[3].0, StyleName::top);
        assert_eq!(*rules[3].1.downcast_ref::<f64>().unwrap(), 2.3 as f32 as f64);
        assert_eq!(rules[4].0, StyleName::width);
        assert_eq!(*rules[4].1.downcast_ref::<f64>().unwrap(), 4. as f32 as f64);
        assert_eq!(rules[5].0, StyleName::height);
        assert_eq!(*rules[5].1.downcast_ref::<f64>().unwrap(), 5. as f32 as f64);
        assert_eq!(rules[6].0, StyleName::font_family);
        assert_eq!(rules[6].1.downcast_ref::<String>().unwrap().clone(), "\"宋体\",\"sans\"");
        assert_eq!(rules[7].0, StyleName::font_size);
        assert_eq!(*rules[7].1.downcast_ref::<f32>().unwrap(), 6.);
        assert_eq!(rules[8].0, StyleName::line_height);
        assert_eq!(*rules[8].1.downcast_ref::<f32>().unwrap(), 7.);
        assert_eq!(rules[9].0, StyleName::color);
        assert_eq!(rules[9].1.downcast_ref::<(f32, f32, f32, f32)>().unwrap().clone(), (1., 0., 0., 1.));
        assert_eq!(rules[10].0, StyleName::background_color);
        assert_eq!(rules[10].1.downcast_ref::<(f32, f32, f32, f32)>().unwrap().clone(), (0., 1., 0., 1.));
        assert_eq!(rules[11].0, StyleName::opacity);
        assert_eq!(rules[11].1.downcast_ref::<f32>().unwrap().clone(), 0.8);
    }
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
        let classes = ssg.query_declarations("a");
        assert_eq!(classes.len(), 3);
        assert_eq!(classes[0].iter_rules().next().unwrap().0, StyleName::position);
        assert_eq!(classes[1].iter_rules().next().unwrap().0, StyleName::display);
        assert_eq!(classes[2].iter_rules().next().unwrap().0, StyleName::left);
    }
}
