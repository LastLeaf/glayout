#![allow(non_snake_case)]

use std::any::Any;
use glayout_element_style_macro::*;
use super::*;

style_value_syntax! {
    "display": display(Enum {
        "none" => DisplayType::None,
        "block" => DisplayType::Block,
        "inline" => DisplayType::Inline,
        "inline-block" => DisplayType::InlineBlock,
        "flex" => DisplayType::Flex,
    });

    "position": position(Enum {
        "static" => PositionType::Static,
        "relative" => PositionType::Relative,
        "absolute" => PositionType::Absolute,
        "fixed" => PositionType::Fixed,
        "sticky" => PositionType::Sticky,
    });
    "left": left(LengthF64);
    "top": top(LengthF64);
    "right": right(LengthF64);
    "bottom": bottom(LengthF64);
    "width": width(LengthF64);
    "height": height(LengthF64);

    "font-family": font_family(FontFamily);
    "font-size": font_size(LengthF32);
    "line-height": line_height(LengthF32);
    "font": [font_size "/" line_height font_family | font_size font_family];
    "text-align": text_align(Enum {
        "left" => TextAlignType::Left,
        "center" => TextAlignType::Center,
        "right" => TextAlignType::Right,
    });

    "color": color(Color);
    "background-color": background_color(Color);
    "background": [background_color];
    "opacity": opacity(Number);

    "margin-left": margin_left(LengthF64);
    "margin-right": margin_right(LengthF64);
    "margin-top": margin_top(LengthF64);
    "margin-bottom": margin_bottom(LengthF64);
    "margin": [margin_top margin_right margin_bottom margin_left] | around;
    "padding-left": padding_left(LengthF64);
    "padding-right": padding_right(LengthF64);
    "padding-top": padding_top(LengthF64);
    "padding-bottom": padding_bottom(LengthF64);
    "padding": [padding_top padding_right padding_bottom padding_left] | around;
    "box-sizing": box_sizing(Enum {
        "content-box" => BoxSizingType::ContentBox,
        "padding-box" => BoxSizingType::PaddingBox,
        "border-box" => BoxSizingType::BorderBox,
    });

    "border-left-width": border_left_width(LengthF64);
    "border-left-style": border_left_style(Enum {
        "none" => BorderStyleType::None,
        "solid" => BorderStyleType::Solid,
    });
    "border-left-color": border_left_color(Color);
    "border-right-width": border_right_width(LengthF64);
    "border-right-style": border_right_style(Enum {
        "none" => BorderStyleType::None,
        "solid" => BorderStyleType::Solid,
    });
    "border-right-color": border_right_color(Color);
    "border-top-width": border_top_width(LengthF64);
    "border-top-style": border_top_style(Enum {
        "none" => BorderStyleType::None,
        "solid" => BorderStyleType::Solid,
    });
    "border-top-color": border_top_color(Color);
    "border-bottom-width": border_bottom_width(LengthF64);
    "border-bottom-style": border_bottom_style(Enum {
        "none" => BorderStyleType::None,
        "solid" => BorderStyleType::Solid,
    });
    "border-bottom-color": border_bottom_color(Color);
    "border-left": [border_left_width border_left_style border_left_color];
    "border-right": [border_right_width border_right_style border_right_color];
    "border-top": [border_top_width border_top_style border_top_color];
    "border-bottom": [border_bottom_width border_bottom_style border_bottom_color];
    "border-width": [border_top_width border_right_width border_bottom_width border_left_width] | around;
    "border-style": [border_top_style border_right_style border_bottom_style border_left_style] | around;
    "border-color": [border_top_color border_right_color border_bottom_color border_left_color] | around;
    "border": [
        border_top_width border_top_style border_top_color,
        border_right_width border_right_style border_right_color,
        border_bottom_width border_bottom_style border_bottom_color,
        border_left_width border_left_style border_left_color
    ] | border_around;

    "flex-basis": flex_basis(LengthF64);
    "flex-grow": flex_grow(Number);
    "flex-shrink": flex_shrink(Number);
    "flex": [flex_grow flex_shrink flex_basis];
}

// base parsers
mod ParseBase {
    use super::*;

    pub fn r#Enum<'a, 'b, T: Clone + Send + Sized>(parser: &mut Parser<'a, '_>, mapping: &'b [(&'b str, T)]) -> Result<Box<T>, ParseError<'a, ()>> {
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

    pub fn Number<'a>(parser: &mut Parser<'a, '_>) -> Result<Box<f32>, ParseError<'a, ()>> {
        {
            let r = parser.next();
            if r.is_ok() {
                let token = r.unwrap();
                match token {
                    Token::Number {value, has_sign: _, int_value: _} => {
                        let num: f32 = *value;
                        return Ok(Box::new(num.clone()));
                    },
                    _ => { }
                }
            }
        }
        Err(parser.new_custom_error(()))
    }

    fn Length<'a, T: From<f32> + Clone + Send + Sized>(parser: &mut Parser<'a, '_>) -> Result<Box<T>, ParseError<'a, ()>> {
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
    pub fn LengthF64<'a>(parser: &mut Parser<'a, '_>) -> Result<Box<f64>, ParseError<'a, ()>> {
        Length::<f64>(parser)
    }
    pub fn LengthF32<'a>(parser: &mut Parser<'a, '_>) -> Result<Box<f32>, ParseError<'a, ()>> {
        Length::<f32>(parser)
    }

    pub fn Color<'a>(parser: &mut Parser<'a, '_>) -> Result<Box<(f32, f32, f32, f32)>, ParseError<'a, ()>> {
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

    pub fn FontFamily<'a>(parser: &mut Parser<'a, '_>) -> Result<Box<String>, ParseError<'a, ()>> {
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
                                    let mut name = String::new() + "\"" + s;
                                    if !parser.is_exhausted() {
                                        let token = parser.next().map_err(|_| ())?;
                                        match token {
                                            Token::Ident(s) => {
                                                name = name + " " + s;
                                            },
                                            _ => {
                                                return Err(());
                                            }
                                        }
                                    }
                                    ret.push(name + "\"");
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
}

// post processors
mod ParsePost {
    use super::*;

    macro_rules! arround {
        ($class:expr, $top:ident, $right:ident, $bottom:ident, $left:ident) => {
            let $right = match $right.1.clone() {
                Some(_) => $right,
                None => $top.clone(),
            };
            let $bottom = match $bottom.1.clone() {
                Some(_) => $bottom,
                None => $top.clone(),
            };
            let $left = match $left.1.clone() {
                Some(_) => $left,
                None => $right.clone(),
            };
            parse_append_rule($class, $top.0, $top.1.unwrap());
            parse_append_rule($class, $right.0, $right.1.unwrap());
            parse_append_rule($class, $bottom.0, $bottom.1.unwrap());
            parse_append_rule($class, $left.0, $left.1.unwrap());
        }
    }

    pub fn around<T: 'static + Clone + Send + Sized>(class: &mut ElementClass, top: (StyleName, Option<Box<T>>), right: (StyleName, Option<Box<T>>), bottom: (StyleName, Option<Box<T>>), left: (StyleName, Option<Box<T>>)) {
        arround!(class, top, right, bottom, left);
    }

    pub fn border_around<T: 'static + Clone + Send + Sized>(class: &mut ElementClass,
        top_width: (StyleName, Option<Box<T>>), top_style: (StyleName, Option<Box<BorderStyleType>>), top_color: (StyleName, Option<Box<(f32, f32, f32, f32)>>), _: (StyleName, Option<Box<T>>),
        right_width: (StyleName, Option<Box<T>>), right_style: (StyleName, Option<Box<BorderStyleType>>), right_color: (StyleName, Option<Box<(f32, f32, f32, f32)>>), _: (StyleName, Option<Box<T>>),
        bottom_width: (StyleName, Option<Box<T>>), bottom_style: (StyleName, Option<Box<BorderStyleType>>), bottom_color: (StyleName, Option<Box<(f32, f32, f32, f32)>>), _: (StyleName, Option<Box<T>>),
        left_width: (StyleName, Option<Box<T>>), left_style: (StyleName, Option<Box<BorderStyleType>>), left_color: (StyleName, Option<Box<(f32, f32, f32, f32)>>)
    ) {
        arround!(class, top_width, right_width, bottom_width, left_width);
        if top_style.1.is_some() {
            arround!(class, top_style, right_style, bottom_style, left_style);
        }
        if top_color.1.is_some() {
            arround!(class, top_color, right_color, bottom_color, left_color);
        }
    }
}

// helpers
fn parse_str<'a, 'b>(parser: &mut Parser<'a, '_>, s: &'b str) -> Result<(), ParseError<'a, ()>> {
    {
        let r = parser.next();
        if r.is_ok() {
            let token = r.unwrap();
            match token {
                Token::Delim(c) => {
                    return if s.chars().count() == 1 && s.chars().next().unwrap() == *c {
                        Ok(())
                    } else {
                        Err(parser.new_custom_error(()))
                    };
                },
                _ => {
                    return Err(parser.new_custom_error(()));
                }
            }
        }
    }
    Err(parser.new_custom_error(()))
}
#[inline]
fn parse_into_any_pair(v: (StyleName, Option<Box<dyn Any + Send>>)) -> (StyleName, Option<Box<dyn Any + Send>>) {
    v
}
#[inline]
fn parse_append_rule(class: &mut ElementClass, style_name: StyleName, v: Box<dyn Any + Send>) {
    class.add_rule(style_name, v);
}
#[inline]
fn parse_fail<'a>(e: &ParseError<'a, ()>) {
    warn!("CSS Parsing {:?}", e);
}
