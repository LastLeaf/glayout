use std::rc::Rc;
use std::any::Any;
use cssparser::{Delimiter, Token, ParserInput, Parser, ParseError, Color};
use std::collections::HashMap;
pub(self) use super::*;

mod selector;
use self::selector::{Selector, SelectorFragment, SelectorQuery};
mod rule;
use self::rule::Rule;

type ValueParsingResult<'a> = Result<Box<Any + Send>, ParseError<'a, ()>>;

pub struct StyleSheet {
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
            if parser.try(|parser| {
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
            if parser.try(|parser| {
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
        const TRANSPARENT_COLOR: (f32, f32, f32, f32) = (0., 0., 0., 0.);
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
                                let e: &ParseError<()> = &e;
                                warn!("CSS ParseError {:?}", e);
                            }
                        };
                    }
                }
                macro_rules! add_border_rule {
                    ($width:ident, $color:ident, $v:expr) => {
                        add_rule!(StyleName::$width, $v.0);
                        let enabled = match $v.1 {
                            Err(_) => false,
                            Ok(v) => *v.downcast_ref::<bool>().unwrap(),
                        };
                        if enabled {
                            add_rule!(StyleName::$color, $v.2);
                        } else {
                            add_rule!(StyleName::$color, Ok(Box::new(TRANSPARENT_COLOR)));
                        }
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
                    "right" => {
                        add_rule!(StyleName::right, Self::parse_length::<f64>(parser));
                    },
                    "bottom" => {
                        add_rule!(StyleName::bottom, Self::parse_length::<f64>(parser));
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
                    "text-align" => {
                        const MAPPING: [(&'static str, TextAlignType); 3] = [
                            ("left", TextAlignType::Left),
                            ("center", TextAlignType::Center),
                            ("right", TextAlignType::Right),
                        ];
                        add_rule!(StyleName::text_align, Self::parse_enum(parser, &MAPPING));
                    },
                    "color" => {
                        add_rule!(StyleName::color, Self::parse_color(parser));
                    },
                    "background" => {
                        add_rule!(StyleName::background_color, Self::parse_color(parser));
                    },
                    "background-color" => {
                        add_rule!(StyleName::background_color, Self::parse_color(parser));
                    },
                    "opacity" => {
                        add_rule!(StyleName::opacity, Self::parse_number::<f32>(parser));
                    },
                    // "transform" => {
                    //     unimplemented!();
                    // },
                    "margin" => {
                        let [top, right, bottom, left] = Self::parse_bounds::<f64>(parser);
                        add_rule!(StyleName::margin_left, left);
                        add_rule!(StyleName::margin_right, right);
                        add_rule!(StyleName::margin_top, top);
                        add_rule!(StyleName::margin_bottom, bottom);
                    },
                    "margin-left" => {
                        add_rule!(StyleName::margin_left, Self::parse_length::<f64>(parser));
                    },
                    "margin-right" => {
                        add_rule!(StyleName::margin_right, Self::parse_length::<f64>(parser));
                    },
                    "margin-top" => {
                        add_rule!(StyleName::margin_top, Self::parse_length::<f64>(parser));
                    },
                    "margin-bottom" => {
                        add_rule!(StyleName::margin_bottom, Self::parse_length::<f64>(parser));
                    },
                    "padding" => {
                        let [top, right, bottom, left] = Self::parse_bounds::<f64>(parser);
                        add_rule!(StyleName::padding_left, left);
                        add_rule!(StyleName::padding_right, right);
                        add_rule!(StyleName::padding_top, top);
                        add_rule!(StyleName::padding_bottom, bottom);
                    },
                    "padding-left" => {
                        add_rule!(StyleName::padding_left, Self::parse_length::<f64>(parser));
                    },
                    "padding-right" => {
                        add_rule!(StyleName::padding_right, Self::parse_length::<f64>(parser));
                    },
                    "padding-top" => {
                        add_rule!(StyleName::padding_top, Self::parse_length::<f64>(parser));
                    },
                    "padding-bottom" => {
                        add_rule!(StyleName::padding_bottom, Self::parse_length::<f64>(parser));
                    },
                    "box-sizing" => {
                        const MAPPING: [(&'static str, BoxSizingType); 3] = [
                            ("content-box", BoxSizingType::ContentBox),
                            ("padding-box", BoxSizingType::PaddingBox),
                            ("border-box", BoxSizingType::BorderBox),
                        ];
                        add_rule!(StyleName::text_align, Self::parse_enum(parser, &MAPPING));
                    },
                    "border" => {
                        let [top, right, bottom, left] = Self::parse_border_multi::<f64>(parser);
                        add_border_rule!(border_left_width, border_left_color, left);
                        add_border_rule!(border_right_width, border_right_color, right);
                        add_border_rule!(border_top_width, border_top_color, top);
                        add_border_rule!(border_bottom_width, border_bottom_color, bottom);
                    },
                    "border-width" => {
                        let [top, right, bottom, left] = Self::parse_bounds::<f64>(parser);
                        add_rule!(StyleName::border_left_width, left);
                        add_rule!(StyleName::border_right_width, right);
                        add_rule!(StyleName::border_top_width, top);
                        add_rule!(StyleName::border_bottom_width, bottom);
                    },
                    "border-color" => {
                        match Self::parse_color_inner(parser) {
                            Ok(v) => {
                                add_rule!(StyleName::border_left_color, Ok(v.clone()));
                                add_rule!(StyleName::border_right_color, Ok(v.clone()));
                                add_rule!(StyleName::border_top_color, Ok(v.clone()));
                                add_rule!(StyleName::border_bottom_color, Ok(v));
                            },
                            Err(e) => {
                                add_rule!(StyleName::border_left_color, Err(e.clone()));
                                add_rule!(StyleName::border_right_color, Err(e.clone()));
                                add_rule!(StyleName::border_top_color, Err(e.clone()));
                                add_rule!(StyleName::border_bottom_color, Err(e));
                            }
                        }
                    },
                    "border-style" => {
                        match Self::parse_border_type_inner(parser) {
                            Ok(enabled) => {
                                if *enabled {
                                    // FIXME impl real border-style
                                } else {
                                    add_rule!(StyleName::border_left_color, Ok(Box::new(TRANSPARENT_COLOR)));
                                    add_rule!(StyleName::border_right_color, Ok(Box::new(TRANSPARENT_COLOR)));
                                    add_rule!(StyleName::border_top_color, Ok(Box::new(TRANSPARENT_COLOR)));
                                    add_rule!(StyleName::border_bottom_color, Ok(Box::new(TRANSPARENT_COLOR)));
                                }
                            },
                            Err(e) => {
                                add_rule!(StyleName::border_left_color, Err(e.clone()));
                                add_rule!(StyleName::border_right_color, Err(e.clone()));
                                add_rule!(StyleName::border_top_color, Err(e.clone()));
                                add_rule!(StyleName::border_bottom_color, Err(e));
                            }
                        }
                    },
                    "border-left" => {
                        let (width, style, color) = Self::parse_border_single::<f64>(parser);
                        add_rule!(StyleName::border_left_width, width);
                        let enabled = match style {
                            Err(_) => false,
                            Ok(v) => *v.downcast_ref::<bool>().unwrap(),
                        };
                        if enabled {
                            add_rule!(StyleName::border_left_color, color);
                        } else {
                            add_rule!(StyleName::border_left_color, Ok(Box::new(TRANSPARENT_COLOR)));
                        }
                    },
                    "border-right" => {
                        let (width, style, color) = Self::parse_border_single::<f64>(parser);
                        add_rule!(StyleName::border_right_width, width);
                        let enabled = match style {
                            Err(_) => false,
                            Ok(v) => *v.downcast_ref::<bool>().unwrap(),
                        };
                        if enabled {
                            add_rule!(StyleName::border_right_color, color);
                        } else {
                            add_rule!(StyleName::border_right_color, Ok(Box::new(TRANSPARENT_COLOR)));
                        }
                    },
                    "border-top" => {
                        let (width, style, color) = Self::parse_border_single::<f64>(parser);
                        add_rule!(StyleName::border_top_width, width);
                        let enabled = match style {
                            Err(_) => false,
                            Ok(v) => *v.downcast_ref::<bool>().unwrap(),
                        };
                        if enabled {
                            add_rule!(StyleName::border_top_color, color);
                        } else {
                            add_rule!(StyleName::border_top_color, Ok(Box::new(TRANSPARENT_COLOR)));
                        }
                    },
                    "border-bottom" => {
                        let (width, style, color) = Self::parse_border_single::<f64>(parser);
                        add_rule!(StyleName::border_bottom_width, width);
                        let enabled = match style {
                            Err(_) => false,
                            Ok(v) => *v.downcast_ref::<bool>().unwrap(),
                        };
                        if enabled {
                            add_rule!(StyleName::border_bottom_color, color);
                        } else {
                            add_rule!(StyleName::border_bottom_color, Ok(Box::new(TRANSPARENT_COLOR)));
                        }
                    },
                    "border-left-width" => {
                        add_rule!(StyleName::border_left_width, Self::parse_length::<f64>(parser));
                    },
                    "border-right-width" => {
                        add_rule!(StyleName::border_right_width, Self::parse_length::<f64>(parser));
                    },
                    "border-top-width" => {
                        add_rule!(StyleName::border_top_width, Self::parse_length::<f64>(parser));
                    },
                    "border-bottom-width" => {
                        add_rule!(StyleName::border_bottom_width, Self::parse_length::<f64>(parser));
                    },
                    "border-left-color" => {
                        add_rule!(StyleName::border_left_color, Self::parse_color(parser));
                    },
                    "border-right-color" => {
                        add_rule!(StyleName::border_right_color, Self::parse_color(parser));
                    },
                    "border-top-color" => {
                        add_rule!(StyleName::border_top_color, Self::parse_color(parser));
                    },
                    "border-bottom-color" => {
                        add_rule!(StyleName::border_bottom_color, Self::parse_color(parser));
                    },
                    "flex" => {
                        let [grow, shrink] = Self::parse_flex::<f32>(parser);
                        add_rule!(StyleName::flex_grow, grow);
                        add_rule!(StyleName::flex_shrink, shrink);
                    },
                    "flex-grow" => {
                        add_rule!(StyleName::flex_grow, Self::parse_length::<f32>(parser));
                    },
                    "flex-shrink" => {
                        add_rule!(StyleName::flex_shrink, Self::parse_length::<f32>(parser));
                    },
                    _ => {
                        add_rule!(StyleName::glayout_unrecognized, Err(parser.new_custom_error::<_, ()>(())));
                    }
                };
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
            }).unwrap();
        }
        Ok(())
    }
    #[inline]
    fn parse_number<'a, T: 'static + From<f32> + Send + Sync + Clone>(parser: &mut Parser<'a, '_>) -> ValueParsingResult<'a> {
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
    fn parse_length_inner<'a, T: 'static + From<f32> + Send + Sync + Clone>(parser: &mut Parser<'a, '_>) -> Result<Box<T>, ParseError<'a, ()>> {
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
    fn parse_length<'a, T: 'static + From<f32> + Send + Sync + Clone>(parser: &mut Parser<'a, '_>) -> ValueParsingResult<'a> {
        match Self::parse_length_inner::<T>(parser) {
            Err(e) => Err(e),
            Ok(r) => Ok(r)
        }
    }
    fn parse_flex<'a, T: 'static + From<f32> + Send + Sync + Clone>(parser: &mut Parser<'a, '_>) -> [ValueParsingResult<'a>; 2] {
        match Self::parse_length_inner::<T>(parser) {
            Err(e) => [Err(e.clone()), Err(e)],
            Ok(grow) => {
                let next = parser.try(|parser| {
                    Self::parse_length_inner::<T>(parser)
                });
                match next {
                    Err(_) => {
                        [Ok(grow.clone()), Ok(grow)]
                    },
                    Ok(shrink) => {
                        [Ok(grow), Ok(shrink)]
                    },
                }
            }
        }
    }
    #[inline]
    fn parse_bounds<'a, T: 'static + From<f32> + Send + Sync + Clone>(parser: &mut Parser<'a, '_>) -> [ValueParsingResult<'a>; 4] {
        let next = parser.try(|parser| {
            Self::parse_length_inner::<T>(parser)
        });
        match next {
            Err(e) => [Err(e.clone()), Err(e.clone()), Err(e.clone()), Err(e)],
            Ok(next) => {
                let top = next;
                let right;
                let bottom;
                let left;
                let next = parser.try(|parser| {
                    Self::parse_length_inner::<T>(parser)
                });
                match next {
                    Err(_) => {
                        right = top.clone();
                        bottom = top.clone();
                        left = top.clone();
                    },
                    Ok(next) => {
                        right = next;
                        let next = parser.try(|parser| {
                            Self::parse_length_inner::<T>(parser)
                        });
                        match next {
                            Err(_) => {
                                bottom = top.clone();
                                left = right.clone();
                            },
                            Ok(next) => {
                                bottom = next;
                                let next = parser.try(|parser| {
                                    Self::parse_length_inner::<T>(parser)
                                });
                                match next {
                                    Err(_) => {
                                        left = right.clone();
                                    },
                                    Ok(next) => {
                                        left = next;
                                    }
                                }
                            }
                        }
                    }
                }
                [
                Ok(top),
                Ok(right),
                Ok(bottom),
                Ok(left),
                ]
            },
        }
    }
    #[inline]
    fn parse_color_inner<'a>(parser: &mut Parser<'a, '_>) -> Result<Box<(f32, f32, f32, f32)>, ParseError<'a, ()>> {
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
    fn parse_color<'a>(parser: &mut Parser<'a, '_>) -> ValueParsingResult<'a> {
        match Self::parse_color_inner(parser) {
            Err(e) => Err(e),
            Ok(r) => Ok(r)
        }
    }
    #[inline]
    fn parse_border_type_inner<'a>(parser: &mut Parser<'a, '_>) -> Result<Box<bool>, ParseError<'a, ()>> {
        const MAPPING: [(&'static str, bool); 2] = [
            ("none", false),
            ("solid", true),
        ];
        Self::parse_enum_inner(parser, &MAPPING)
    }
    #[inline]
    fn parse_border_type<'a>(parser: &mut Parser<'a, '_>) -> ValueParsingResult<'a> {
        match Self::parse_border_type_inner(parser) {
            Err(e) => Err(e),
            Ok(r) => Ok(r)
        }
    }
    #[inline]
    fn parse_border_single<'a, T: 'static + From<f32> + Send + Sync + Clone>(parser: &mut Parser<'a, '_>) -> (ValueParsingResult<'a>, ValueParsingResult<'a>, ValueParsingResult<'a>) {
        let width = Self::parse_length::<T>(parser);
        let enabled = Self::parse_border_type(parser);
        let color = Self::parse_color(parser);
        (width, enabled, color)
    }
    #[inline]
    fn parse_border_multi<'a, T: 'static + From<f32> + Send + Sync + Clone>(parser: &mut Parser<'a, '_>) -> [(ValueParsingResult<'a>, ValueParsingResult<'a>, ValueParsingResult<'a>); 4] {
        match Self::parse_length_inner::<T>(parser) {
            Err(e) => [
                (Err(e.clone()), Err(e.clone()), Err(e.clone())),
                (Err(e.clone()), Err(e.clone()), Err(e.clone())),
                (Err(e.clone()), Err(e.clone()), Err(e.clone())),
                (Err(e.clone()), Err(e.clone()), Err(e.clone())),
            ],
            Ok(width) => {
                const MAPPING: [(&'static str, bool); 2] = [
                    ("none", false),
                    ("solid", true),
                ];
                match Self::parse_enum_inner(parser, &MAPPING) {
                    Err(e) => [
                        (Ok(width.clone()), Err(e.clone()), Err(e.clone())),
                        (Ok(width.clone()), Err(e.clone()), Err(e.clone())),
                        (Ok(width.clone()), Err(e.clone()), Err(e.clone())),
                        (Ok(width), Err(e.clone()), Err(e)),
                    ],
                    Ok(enabled) => {
                        match Self::parse_color_inner(parser) {
                            Err(e) => [
                                (Ok(width.clone()), Ok(enabled.clone()), Err(e.clone())),
                                (Ok(width.clone()), Ok(enabled.clone()), Err(e.clone())),
                                (Ok(width.clone()), Ok(enabled.clone()), Err(e.clone())),
                                (Ok(width), Ok(enabled), Err(e)),
                            ],
                            Ok(color) => [
                                (Ok(width.clone()), Ok(enabled.clone()), Ok(color.clone())),
                                (Ok(width.clone()), Ok(enabled.clone()), Ok(color.clone())),
                                (Ok(width.clone()), Ok(enabled.clone()), Ok(color.clone())),
                                (Ok(width), Ok(enabled), Ok(color)),
                            ]
                        }
                    }
                }
            }
        }
    }
    #[inline]
    fn _parse_string_or_ident<'a>(parser: &mut Parser<'a, '_>) -> ValueParsingResult<'a> {
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
    fn parse_enum_inner<'a, T: Clone + Send + Sync + Sized>(parser: &mut Parser<'a, '_>, mapping: &'static [(&'static str, T)]) -> Result<Box<T>, ParseError<'a, ()>> {
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
    fn parse_enum<'a, T: Clone + Send + Sync + Sized>(parser: &mut Parser<'a, '_>, mapping: &'static [(&'static str, T)]) -> ValueParsingResult<'a> {
        match Self::parse_enum_inner::<T>(parser, mapping) {
            Err(e) => Err(e),
            Ok(r) => Ok(r)
        }
    }
    #[inline]
    fn parse_font_family<'a>(parser: &mut Parser<'a, '_>) -> ValueParsingResult<'a> {
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
    use std::any::Any;
    use super::{StyleSheet, StyleSheetGroup, StyleName, SelectorQuery};

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
        let classes = ss.query_declarations(&SelectorQuery::new("", "", Box::new(["a"])));
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
        let classes = ssg.query_declarations("", "", Box::new(["a"]));
        assert_eq!(classes.len(), 3);
        assert_eq!(classes[0].iter_rules().next().unwrap().0, StyleName::position);
        assert_eq!(classes[1].iter_rules().next().unwrap().0, StyleName::display);
        assert_eq!(classes[2].iter_rules().next().unwrap().0, StyleName::left);
    }
}
