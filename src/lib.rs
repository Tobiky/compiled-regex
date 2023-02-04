mod types;

use proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn regex(tokens: TokenStream) -> TokenStream {
    return format!(r#"println!("\"{{}}\"", "{}")"#, tokens.to_string()).parse().unwrap()
}
