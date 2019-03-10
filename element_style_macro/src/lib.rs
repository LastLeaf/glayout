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
impl ToTokens for PropertyDefinitionTokens {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, value_type } = self;
        tokens.append_all(quote! {
            #name : StyleValue<#value_type>
        });
    }
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

// into default values
struct PropertyDefaultValueTokens {
    name: Ident,
    default_value_referrer: Ident,
    default_value: Expr,
    inherit: bool,
}
impl ToTokens for PropertyDefaultValueTokens {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, default_value_referrer, default_value, inherit } = self;
        tokens.append_all(quote! {
            #name : StyleValue::new(StyleValueReferrer::#default_value_referrer, #default_value, #inherit)
        });
    }
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

// into struct impl
struct PropertyImplTokens {
    name: Ident,
    value_type: Type,
    default_value_referrer: Ident,
    default_value: Expr,
    layout_dirty: bool,
    inherit: bool,
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

// parent updated impl
struct PropertyParentUpdatedTokens {
    name: Ident,
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
impl Into<Vec<PropertyParentUpdatedTokens>> for Properties {
    fn into(self) -> Vec<PropertyParentUpdatedTokens> {
        self.p.into_iter().map(|p| {
            PropertyParentUpdatedTokens {
                name: p.name,
            }
        }).collect()
    }
}

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
