mod types;
#[macro_use] mod code_gen_util;

use regex_syntax::Parser;

use proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn regex(tokens: TokenStream) -> TokenStream {
    let inp = tokens.to_string();
    let nme = regex_struct_name!(inp.clone());
    return format!(r##"println!("\"{{}}\" = {{}}", r#"{}"#, r#"{}"#)"##, inp, nme).parse().unwrap()
}
