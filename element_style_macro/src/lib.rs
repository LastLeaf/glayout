extern crate proc_macro;
use self::proc_macro::TokenStream;

#[proc_macro]
pub fn style_struct(item: TokenStream) -> TokenStream {
    let ret = (String::new() + "define_struct!(" + &item.to_string() + ");").parse().unwrap();
    ret
}
