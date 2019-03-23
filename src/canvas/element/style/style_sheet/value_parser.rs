#![allow(non_snake_case)]

use std::borrow::Cow;
use std::any::Any;
use glayout_element_style_macro::*;
use super::*;

type ValueParsingResult<'a, T> = Result<Box<StyleValue<T>>, ParseError<'a, ()>>;

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
    use std::ops::Mul;
    use super::*;

    #[inline]
    fn absolute<T: Clone>(t: T) -> StyleValue<T> {
        StyleValue::new(StyleValueReferrer::Absolute, t, false)
    }

    pub(super) fn r#Enum<'a, 'b, T: Clone + Send + Sized>(parser: &mut Parser<'a, '_>, mapping: &'b [(&'b str, T)]) -> ValueParsingResult<'a, T> {
        {
            let r = parser.expect_ident();
            if r.is_ok() {
                let value = r.unwrap();
                for (s, t) in mapping {
                    if value == s {
                        let t: T = (*t).clone();
                        return Ok(Box::new(absolute(t)));
                    }
                }
            }
        }
        Err(parser.new_custom_error(()))
    }

    pub(super) fn Number<'a>(parser: &mut Parser<'a, '_>) -> ValueParsingResult<'a, f32> {
        {
            let r = parser.next();
            if r.is_ok() {
                let token = r.unwrap();
                match token {
                    Token::Number {value, has_sign: _, int_value: _} => {
                        let num: f32 = *value;
                        return Ok(Box::new(absolute(num.clone())));
                    },
                    _ => { }
                }
            }
        }
        Err(parser.new_custom_error(()))
    }

    fn Length<'a, T: From<f32> + Mul + Clone + Send + Sized>(parser: &mut Parser<'a, '_>) -> ValueParsingResult<'a, T> where <T as Mul>::Output: Into<T> {
        {
            let r = parser.next();
            if r.is_ok() {
                let token = r.unwrap();
                match token {
                    Token::Number {value, has_sign: _, int_value: _} => {
                        let num: f32 = *value;
                        let num_t: T = num.into();
                        return Ok(Box::new(absolute(num_t.clone())));
                    },
                    Token::Dimension {value, unit, has_sign: _, int_value: _} => {
                        let num: f32 = *value;
                        let num_t: T = num.into();
                        return match unit.as_ref() {
                            "px" => {
                                Ok(Box::new(absolute(num_t.clone())))
                            },
                            "in" => {
                                Ok(Box::new(absolute((num_t * T::from(96.)).into())))
                            },
                            "cm" => {
                                Ok(Box::new(absolute((num_t * T::from(37.8)).into())))
                            },
                            "mm" => {
                                Ok(Box::new(absolute((num_t * T::from(3.78)).into())))
                            },
                            "em" => {
                                let v = StyleValue::new(StyleValueReferrer::RelativeToParentFontSize, num_t.clone(), false);
                                Ok(Box::new(v))
                            },
                            // NOTE rem has problems on dirty-update
                            // "rem" => {
                            //     let v = StyleValue::new(StyleValueReferrer::RelativeToViewportFontSize, num_t.clone(), false);
                            //     Ok(Box::new(v))
                            // },
                            "vw" => {
                                let v = StyleValue::new(StyleValueReferrer::RelativeToViewportWidth, (num_t * T::from(0.01)).into(), false);
                                Ok(Box::new(v))
                            },
                            "vh" => {
                                let v = StyleValue::new(StyleValueReferrer::RelativeToViewportHeight, (num_t * T::from(0.01)).into(), false);
                                Ok(Box::new(v))
                            },
                            "rpx" => {
                                let v = StyleValue::new(StyleValueReferrer::RelativeToViewportWidth, (num_t * T::from(1. / 750.)).into(), false);
                                Ok(Box::new(v))
                            },
                            _ => {
                                Err(parser.new_custom_error(()))
                            }
                            // NOTE unimplemented ex, ch, vmin, vmax
                        };
                    },
                    Token::Percentage {unit_value, has_sign: _, int_value: _} => {
                        let num: f32 = *unit_value;
                        let num_t: T = num.into();
                        let v = StyleValue::new(StyleValueReferrer::RelativeToParentSize, num_t.clone(), false);
                        return Ok(Box::new(v));
                    },
                    Token::Ident(s) => {
                        return match s.as_ref() {
                            "auto" => {
                                let v = StyleValue::new(StyleValueReferrer::Auto, T::from(0.), false);
                                Ok(Box::new(v))
                            },
                            _ => {
                                Err(parser.new_custom_error(()))
                            }
                        }
                    },
                    _ => { }
                }
            }
        }
        Err(parser.new_custom_error(()))
    }
    pub(super) fn LengthF64<'a>(parser: &mut Parser<'a, '_>) -> ValueParsingResult<'a, f64> {
        Length::<f64>(parser)
    }
    pub(super) fn LengthF32<'a>(parser: &mut Parser<'a, '_>) -> ValueParsingResult<'a, f32> {
        Length::<f32>(parser)
    }

    pub(super) fn Color<'a>(parser: &mut Parser<'a, '_>) -> ValueParsingResult<'a, (f32, f32, f32, f32)> {
        match Color::parse(parser) {
            Ok(c) => {
                match c {
                    Color::RGBA(c) => {
                        Ok(Box::new(absolute((c.red_f32(), c.green_f32(), c.blue_f32(), c.alpha_f32()))))
                    },
                    _ => Err(parser.new_custom_error(()))
                }
            },
            Err(_) => Err(parser.new_custom_error(()))
        }
    }

    pub(super) fn FontFamily<'a>(parser: &mut Parser<'a, '_>) -> ValueParsingResult<'a, Cow<'static, str>> {
        // TODO use Cow<String> for FontFamily
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
                Ok(Box::new(absolute(Cow::from(ret.join(",")))))
            } else {
                Err(r.unwrap_err())
            }
        }
    }
}

// post processors
mod ParsePost {
    use std::fmt::Debug;
    use super::*;
    type OptionValue<T> = Option<Box<StyleValue<T>>>;

    macro_rules! arround {
        ($class:expr, $top:ident, $right:ident, $bottom:ident, $left:ident) => {
            let (top_k, top_v) = $top;
            let (right_k, right_v) = $right;
            let (bottom_k, bottom_v) = $bottom;
            let (left_k, left_v) = $left;
            let top_v = top_v.unwrap();
            let right_v = right_v.unwrap_or_else(|| top_v.clone());
            let bottom_v = bottom_v.unwrap_or_else(|| top_v.clone());
            let left_v = left_v.unwrap_or_else(|| right_v.clone());
            parse_append_rule($class, top_k, top_v);
            parse_append_rule($class, right_k, right_v);
            parse_append_rule($class, bottom_k, bottom_v);
            parse_append_rule($class, left_k, left_v);
        }
    }

    pub(super) fn around<T: 'static + Clone + Send + Sized + Debug>(class: &mut ElementClass, top: (StyleName, Option<Box<T>>), right: (StyleName, Option<Box<T>>), bottom: (StyleName, Option<Box<T>>), left: (StyleName, Option<Box<T>>)) {
        arround!(class, top, right, bottom, left);
    }

    pub(super) fn border_around<T: 'static + Clone + Send + Sized + Debug>(class: &mut ElementClass,
        top_width: (StyleName, Option<Box<T>>), top_style: (StyleName, OptionValue<BorderStyleType>), top_color: (StyleName, OptionValue<(f32, f32, f32, f32)>), _: (StyleName, Option<Box<T>>),
        right_width: (StyleName, Option<Box<T>>), right_style: (StyleName, OptionValue<BorderStyleType>), right_color: (StyleName, OptionValue<(f32, f32, f32, f32)>), _: (StyleName, Option<Box<T>>),
        bottom_width: (StyleName, Option<Box<T>>), bottom_style: (StyleName, OptionValue<BorderStyleType>), bottom_color: (StyleName, OptionValue<(f32, f32, f32, f32)>), _: (StyleName, Option<Box<T>>),
        left_width: (StyleName, Option<Box<T>>), left_style: (StyleName, OptionValue<BorderStyleType>), left_color: (StyleName, OptionValue<(f32, f32, f32, f32)>)
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
