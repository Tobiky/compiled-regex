use compiled_regex_core::regex_syntax::ast::parse::Parser;
use compiled_regex_core::ir;
use compiled_regex_core::ir::IR;

use proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn parse_regex(tokens: TokenStream) -> TokenStream {
    let inp = tokens.to_string();
    let inp = inp[inp.find('"').unwrap() + 1..inp.rfind('"').unwrap()].to_string();
    // let nme = regex_name!(inp.clone());
    match Parser::new().parse(&inp) {
        Ok(ast) => {
            match ir::RegExNode::parse(ast) {
                Ok(reg) => {
                    let types = reg.generate_impl();
                    let implementation = types.iter().next().unwrap();
                    let name = implementation.name.clone();

                    let code = implementation.to_string();

                    let code = format!("use compiled_regex::types::RegExp; #[allow(non_camel_case_types)] #[allow(non_snake_case)] mod __m{} {{{{\n{}\n}}}}",
                                       name,
                                       code);

                    let code = format!(
                        "{}\ntype MyRegex = __m{}::{};",
                        code,
                        name,
                        name);

                    return //format!(r###"println!(r##"{}"##)"###, code)
                        code
                        .replace("{{", "{")
                        .replace("}}", "}")
                        .parse()
                        .unwrap()
                }
                Err(err) => panic!("{:?}", err)
            }
        },
        Err(error) => {
            panic!("{}", error)
        }
    }
}
