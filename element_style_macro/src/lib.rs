#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
use self::proc_macro2::{TokenStream, Span};
use quote::TokenStreamExt;
use quote::quote;
use quote::ToTokens;
use syn::*;
use syn::parse::*;
use syn::punctuated::Punctuated;


// property list parsing
#[derive(Clone)]
struct PropertyDefinition {
    name: Ident,
    value_type: Type,
    default_value_referrer: Ident,
    default_value: Expr,
    layout_dirty: bool,
    inherit: bool,
}
fn has_option(options: &Punctuated<Ident, Token![,]>, name: &str) -> bool {
    options.iter().position(|x| x.to_string() == name).is_some()
}
impl Parse for PropertyDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let value_type = input.parse()?;
        input.parse::<Token![,]>()?;
        let default_value_referrer = input.parse()?;
        let content;
        parenthesized!(content in input);
        let default_value = content.parse()?;
        input.parse::<Token![,]>()?;
        let content;
        parenthesized!(content in input);
        let options = Punctuated::parse_terminated(&content)?;
        Ok(Self {
            name,
            value_type,
            default_value_referrer,
            default_value,
            layout_dirty: has_option(&options, "layout_dirty"),
            inherit: has_option(&options, "inherit"),
        })
    }
}
#[derive(Clone)]
struct Properties {
    p: Punctuated<PropertyDefinition, Token![;]>,
}
impl Parse for Properties {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            p: input.parse_terminated(PropertyDefinition::parse)?,
        })
    }
}

// into struct definition
struct PropertyDefinitionTokens {
    name: Ident,
    value_type: Type,
}
impl Into<Vec<PropertyDefinitionTokens>> for Properties {
    fn into(self) -> Vec<PropertyDefinitionTokens> {
        self.p.into_iter().map(|p| {
            PropertyDefinitionTokens {
                name: p.name,
                value_type: p.value_type
            }
        }).collect()
    }
}
impl ToTokens for PropertyDefinitionTokens {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, value_type } = self;
        tokens.append_all(quote! {
            #name : StyleValue<#value_type>
        });
    }
}

// into default values
struct PropertyDefaultValueTokens {
    name: Ident,
    default_value_referrer: Ident,
    default_value: Expr,
    inherit: bool,
}
impl Into<Vec<PropertyDefaultValueTokens>> for Properties {
    fn into(self) -> Vec<PropertyDefaultValueTokens> {
        self.p.into_iter().map(|p| {
            PropertyDefaultValueTokens {
                name: p.name,
                default_value_referrer: p.default_value_referrer,
                default_value: p.default_value,
                inherit: p.inherit,
            }
        }).collect()
    }
}
impl ToTokens for PropertyDefaultValueTokens {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, default_value_referrer, default_value, inherit } = self;
        tokens.append_all(quote! {
            #name : StyleValue::new(StyleValueReferrer::#default_value_referrer, #default_value, #inherit)
        });
    }
}

// into struct impl
struct PropertyImplTokens {
    name: Ident,
    value_type: Type,
    default_value_referrer: Ident,
    default_value: Expr,
    layout_dirty: bool,
    inherit: bool,
}
impl Into<Vec<PropertyImplTokens>> for Properties {
    fn into(self) -> Vec<PropertyImplTokens> {
        self.p.into_iter().map(|p| {
            PropertyImplTokens {
                name: p.name,
                value_type: p.value_type,
                default_value_referrer: p.default_value_referrer,
                default_value: p.default_value,
                layout_dirty: p.layout_dirty,
                inherit: p.inherit,
            }
        }).collect()
    }
}
impl ToTokens for PropertyImplTokens {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, value_type, default_value_referrer, default_value, layout_dirty, inherit } = self;
        let getter = Ident::new(&(String::from("get_") + &name.to_string()), Span::call_site());
        let setter = Ident::new(&(String::from("set_") + &name.to_string()), Span::call_site());
        let getter_advanced = Ident::new(&(String::from("get_") + &name.to_string() + "_advanced"), Span::call_site());
        let setter_advanced = Ident::new(&(String::from("set_") + &name.to_string() + "_advanced"), Span::call_site());
        let getter_inner = Ident::new(&(String::from("get_") + &name.to_string() + "_inner"), Span::call_site());
        let setter_inner = Ident::new(&(String::from("set_") + &name.to_string() + "_inner"), Span::call_site());
        let update_inherit = Ident::new(&(String::from("update_inherit_") + &name.to_string()), Span::call_site());
        tokens.append_all(quote! {
            impl_style_item!(
                #name,
                #getter,
                #setter,
                #getter_advanced,
                #setter_advanced,
                #getter_inner,
                #setter_inner,
                #update_inherit,
                #value_type,
                StyleValueReferrer::#default_value_referrer,
                #default_value,
                #layout_dirty,
                #inherit
            );
        });
    }
}

// parent updated impl
struct PropertyParentUpdatedTokens {
    name: Ident,
}
impl Into<Vec<PropertyParentUpdatedTokens>> for Properties {
    fn into(self) -> Vec<PropertyParentUpdatedTokens> {
        self.p.into_iter().map(|p| {
            PropertyParentUpdatedTokens {
                name: p.name,
            }
        }).collect()
    }
}
impl ToTokens for PropertyParentUpdatedTokens {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name } = self;
        let update_inherit = Ident::new(&(String::from("update_inherit_") + &name.to_string()), Span::call_site());
        tokens.append_all(quote! {
            impl_parent_updated_item!(self, #name, #update_inherit);
        });
    }
}

// element style struct composer
#[proc_macro]
pub fn element_style(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let props = parse_macro_input!(tokens as Properties);

    let prop_def: Vec<PropertyDefinitionTokens> = props.clone().into();
    let prop_default_value: Vec<PropertyDefaultValueTokens> = props.clone().into();
    let prop_impl: Vec<PropertyImplTokens> = props.clone().into();
    let parent_updated_impl: Vec<PropertyParentUpdatedTokens> = props.clone().into();

    let ret = quote! {
        define_struct!(
            #(#prop_def),*
        );
        define_constructor!(
            #(#prop_default_value),*
        );
        impl_style_list!(
            #(#prop_impl)*
        );
        impl_parent_updated!(
            fn parent_updated(&mut self) {
                #(#parent_updated_impl)*
            }
        );
    };
    proc_macro::TokenStream::from(ret)
}


// style syntax parsing
#[derive(Clone)]
struct StyleSyntaxItem {
    name: LitStr,
    value: StyleSyntaxValue,
    post_processor: Option<Ident>,
}
#[derive(Clone)]
enum StyleSyntaxValue {
    Basic(Ident, StyleSyntaxValueType),
    Combination(Punctuated<Vec<StyleSyntaxValueField>, Token![|]>),
}
#[derive(Clone)]
enum StyleSyntaxValueType {
    Enum(Punctuated<(LitStr, Expr), Token![,]>),
    Other(Ident),
}
#[derive(Clone)]
enum StyleSyntaxValueField {
    SubField(Ident),
    Str(String),
}
impl Parse for StyleSyntaxItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let lookahead = input.lookahead1();
        let value = if lookahead.peek(Ident) {
            let prop_name = input.parse()?;
            let content;
            parenthesized!(content in input);
            let input = content;
            let prop_type_name = input.parse::<Ident>()?;
            let prop_type = match prop_type_name.to_string().as_str() {
                "Enum" => {
                    let content;
                    braced!(content in input);
                    let branches = Punctuated::parse_terminated_with(&content, |input| {
                        let str_name = input.parse()?;
                        input.parse::<Token![=>]>()?;
                        let expr = input.parse()?;
                        Ok((str_name, expr))
                    })?;
                    StyleSyntaxValueType::Enum(branches)
                },
                _ => {
                    StyleSyntaxValueType::Other(prop_type_name)
                },
            };
            StyleSyntaxValue::Basic(
                prop_name,
                prop_type
            )
        } else {
            let content;
            bracketed!(content in input);
            let combination = Punctuated::parse_terminated_with(&content, |input| {
                let mut ret = vec![];
                while !input.is_empty() {
                    ret.push({
                        let lookahead = input.lookahead1();
                        if lookahead.peek(Ident) {
                            StyleSyntaxValueField::SubField(input.parse()?)
                        } else if lookahead.peek(Token![|]) {
                            return Ok(ret);
                        } else if lookahead.peek(Token![,]) {
                            input.parse::<Token![,]>()?;
                            StyleSyntaxValueField::Str(String::from(","))
                        } else if lookahead.peek(LitStr) {
                            StyleSyntaxValueField::Str(input.parse::<LitStr>()?.value())
                        } else {
                            return Err(lookahead.error());
                        }
                    });
                }
                Ok(ret)
            })?;
            StyleSyntaxValue::Combination(combination)
        };
        let lookahead = input.lookahead1();
        let post_processor = if lookahead.peek(Token![|]) {
            input.parse::<Token![|]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self {
            name,
            value,
            post_processor,
        })
    }
}
#[derive(Clone)]
struct StyleSyntaxDefinition {
    items: Punctuated<StyleSyntaxItem, Token![;]>
}
impl Parse for StyleSyntaxDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            items: input.parse_terminated(StyleSyntaxItem::parse)?,
        })
    }
}

// basic style function generation
struct StyleSyntaxFunctions {
    name: Ident,
    value_type: StyleSyntaxValueType,
}
impl Into<Vec<StyleSyntaxFunctions>> for StyleSyntaxDefinition {
    fn into(self) -> Vec<StyleSyntaxFunctions> {
        let mut ret = vec![];
        self.items.into_iter().for_each(|x| {
            match x.value {
                StyleSyntaxValue::Basic(name, value_type) => {
                    ret.push(StyleSyntaxFunctions {
                        name,
                        value_type,
                    });
                },
                _ => { }
            }
        });
        ret
    }
}
impl ToTokens for StyleSyntaxFunctions {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, value_type } = self;
        match value_type {
            StyleSyntaxValueType::Enum(list) => {
                let list = list.iter().map(|(n, e)| {
                    quote! {
                        (#n, #e)
                    }
                });
                tokens.append_all(quote! {
                    let #name = |parser: &mut Parser<'a, '_>| {
                        let mapping = [
                            #(#list),*
                        ];
                        ParseBase::Enum(parser, &mapping)
                    };
                });
            },
            StyleSyntaxValueType::Other(base_type) => {
                tokens.append_all(quote! {
                    let #name = |parser: &mut Parser<'a, '_>| {
                        ParseBase::#base_type(parser)
                    };
                });
            },
        }
    }
}

// style parsing map generation
struct StyleSyntaxMap {
    name: LitStr,
    value: StyleSyntaxValue,
    post_processor: Option<Ident>,
}
impl Into<Vec<StyleSyntaxMap>> for StyleSyntaxDefinition {
    fn into(self) -> Vec<StyleSyntaxMap> {
        self.items.into_iter().map(|x| {
            StyleSyntaxMap {
                name: x.name,
                value: x.value,
                post_processor: x.post_processor,
            }
        }).collect()
    }
}
impl ToTokens for StyleSyntaxMap {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, value, post_processor } = self;
        match value {
            StyleSyntaxValue::Basic(prop_name, _) => {
                let post = match post_processor {
                    None => {
                        quote! {
                            parse_append_rule(class, StyleName::#prop_name, v)
                        }
                    },
                    Some(x) => {
                        quote! {
                            ParsePost::#x(StyleName::#prop_name, v);
                        }
                    }
                };
                tokens.append_all(quote! {
                    #name => {
                        match #prop_name(parser) {
                            Ok(v) => {
                                #post
                            },
                            Err(e) => {
                                parse_fail(&e);
                            }
                        };
                    }
                });
            },
            StyleSyntaxValue::Combination(list) => {
                let list: Vec<TokenStream> = list.iter().map(|x| {
                    let c: Vec<TokenStream> = x.iter().map(|x| {
                        match x {
                            StyleSyntaxValueField::SubField(prop_name) => {
                                quote! {
                                    {
                                        if parser.is_exhausted() {
                                            (StyleName::#prop_name, None)
                                        } else {
                                            match #prop_name(parser) {
                                                Ok(x) => {
                                                    (StyleName::#prop_name, Some(x))
                                                },
                                                Err(_) => {
                                                    return Err(());
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            StyleSyntaxValueField::Str(s) => {
                                quote! {
                                    {
                                        if !parser.is_exhausted() {
                                            parse_str(parser, #s).map_err(|_| ())?;
                                        }
                                        (StyleName::glayout_unrecognized, None)
                                    }
                                }
                            },
                        }
                    }).collect();
                    let post = match post_processor {
                        None => {
                            quote! {
                                let mut r = [#( parse_into_any_pair(#c) ),*];
                                for item in r.iter_mut() {
                                    let v = item.1.take();
                                    match v {
                                        Some(v) => {
                                            parse_append_rule(class, item.0, v);
                                        },
                                        None => { }
                                    };
                                }
                            }
                        },
                        Some(x) => {
                            quote! {
                                ParsePost::#x(class, #(#c),*);
                            }
                        }
                    };
                    quote! {
                        match parser.r#try(|parser| {
                            #post
                            Ok(())
                        }) {
                            Ok(_) => { return },
                            Err(_) => { }
                        };
                    }
                }).collect();
                tokens.append_all(quote! {
                    #name => {
                        if !parser.is_exhausted() {
                            #(#list)*
                        }
                        parse_fail(&parser.new_custom_error::<_, ()>(()));
                    }
                });
            },
        }
    }
}

// style sheet parser struct composer
#[proc_macro]
pub fn style_value_syntax(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let style_syntax = parse_macro_input!(tokens as StyleSyntaxDefinition);

    let style_syntax_functions: Vec<StyleSyntaxFunctions> = style_syntax.clone().into();
    let style_syntax_map: Vec<StyleSyntaxMap> = style_syntax.clone().into();

    let ret = quote! {
        pub fn parse_value<'a>(parser: &mut Parser<'a, '_>, class: &mut ElementClass, str_name: &str) {
            #(#style_syntax_functions)*
            match str_name {
                #(#style_syntax_map),*,
                _ => {
                    parse_fail(&parser.new_custom_error::<_, ()>(()));
                },
            }
        }
    };
    proc_macro::TokenStream::from(ret)
    // panic!(proc_macro::TokenStream::from(ret).to_string())
}
