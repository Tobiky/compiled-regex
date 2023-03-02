use compiled_regex_core::regex_syntax::ast::parse::Parser;
use compiled_regex_core::ir;
use compiled_regex_core::ir::IR;

use proc_macro::{self, TokenTree};
use proc_macro::TokenStream;

fn parse_regex_string(regex: &str) -> String {
    // Try to parse the regex
    match Parser::new().parse(regex) {
        Ok(ast) => {
            // Try to parse the regex AST and generate code
            match ir::RegExNode::parse(&ast) {
                Ok(reg) => {
                    // Generate the implementation, get the name, and turn the impl into a string
                    let impls = reg.generate_impl();
                    let implementation = impls.iter().next().unwrap();
                    let name = implementation.name.clone();

                    let code = implementation.to_string();

                    // Surround the implementation with a module
                    let code = format!("use compiled_regex::types::RegExp; #[allow(non_camel_case_types)] #[allow(non_snake_case)] mod __m{} {{{{\n{}\n}}}}",
                                       name,
                                       code);

                    // Temporary anchor for the generated final type
                    let code = format!(
                        "{}\ntype Regex = __m{}::{};",
                        code,
                        name,
                        name);

                    return
                        // // Generate code to print the rust code
                        // format!(r###"println!(r##"{}"##);"###, code)
                        // Generate plain code (braces have to be normalized)
                        code
                        .replace("{{", "{")
                        .replace("}}", "}")
                }
                // Parsing AST erred, panic and GTFO
                Err(err) => panic!("{:?}", err)
            }
        },
        // Could not parse the RegEx
        Err(error) => {
            panic!("{}", error)
        }
    }
}

#[proc_macro]
pub fn parse_regex(tokens: TokenStream) -> TokenStream {
    // Get the regex within the quotes
    let inp = tokens.to_string();
    let inp = inp[inp.find('"').unwrap() + 1..inp.rfind('"').unwrap()].to_string();

    // Parse the RegEx into actual code
    let code = parse_regex_string(&inp);

    // Parse the code into Rust tokens
    return code.parse().unwrap()
}
