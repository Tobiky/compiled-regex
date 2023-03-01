use compiled_regex_core::regex_syntax::ast::parse::Parser;
use compiled_regex_core::ir;
use compiled_regex_core::ir::IR;

use proc_macro::{self, TokenTree};
use proc_macro::TokenStream;

fn parse_regex_string(regex: &str) -> String {
    match Parser::new().parse(regex) {
        Ok(ast) => {
            match ir::RegExNode::parse(&ast) {
                Ok(reg) => {
                    let types = reg.generate_impl();
                    let implementation = types.iter().next().unwrap();
                    let name = implementation.name.clone();

                    let code = implementation.to_string();

                    let code = format!("use compiled_regex::types::RegExp; #[allow(non_camel_case_types)] #[allow(non_snake_case)] mod __m{} {{{{\n{}\n}}}}",
                                       name,
                                       code);

                    let code = format!(
                        "{}\ntype Regex = __m{}::{};",
                        code,
                        name,
                        name);

                    return // format!(r###"println!(r##"{}"##);"###, code)
                        code
                        .replace("{{", "{")
                        .replace("}}", "}")
                }
                Err(err) => panic!("{:?}", err)
            }
        },
        Err(error) => {
            panic!("{}", error)
        }
    }
}

#[proc_macro]
pub fn parse_regex(tokens: TokenStream) -> TokenStream {
    let inp = tokens.to_string();
    let inp = inp[inp.find('"').unwrap() + 1..inp.rfind('"').unwrap()].to_string();

    let code = parse_regex_string(&inp);

    return code.parse().unwrap()
}
