use compiled_regex_core::regex_syntax::ast::parse::Parser;
use compiled_regex_core::ir;
use compiled_regex_core::ir::IR;

use compiled_regex_core::types::CompileError;
use proc_macro::{self, TokenTree};
use proc_macro::TokenStream;

use litrs::StringLit;

fn parse_regex_string(export_name: &str, regex: &str) -> String {
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
                        "{}\ntype {} = __m{}::{};",
                        code,
                        export_name,
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

fn parse_token_stream(tokens: TokenStream) -> Result<(String, String), CompileError> {
    if tokens.is_empty() {
        // TODO: Should return error describing empty token set
        return Err(CompileError::UnexpectedToken(0, 0))
    }

    let mut iter = tokens.into_iter();

    // Find the name (an identity or string literal) for the RegEx
    let name = match iter.next() {
        Some(TokenTree::Ident(x)) => x.to_string(),
        Some(TokenTree::Literal(x)) => {
            if let Ok(s) = StringLit::try_from(x) {
                s.to_string()
            } else {
                // TODO: Specify illegal literal type usage
                return Err(CompileError::UnexpectedToken(0, 0))
            }
        }
        // TODO: Specifiy that identity or name is needed
        _ => return Err(CompileError::UnexpectedToken(0, 0))
    };

    // Make sure there is a delimiter inbetween
    // Can possibly skip this by just making sure that the the last
    // item is correct
    if !matches!(iter.next(), Some(TokenTree::Punct(_))) {
        // TODO: Specify that some punctuation is needed
        // currently can be any, not sure if it should be forced
        // to anything specific.
        return Err(CompileError::UnexpectedToken(0, 0))
    }

    // Get the regex string literal
    let regex = match iter.next() {
        Some(TokenTree::Literal(x)) => {
            if let Ok(s) = StringLit::try_from(x) {
                s.to_string()
            } else {
                // TODO: Specify illegal literal type usage
                return Err(CompileError::UnexpectedToken(0, 0))
            }
        }
        // TODO: Specifiy that str literal is needed
        _ => return Err(CompileError::UnexpectedToken(0, 0))
    };

    // Strip surrounding string marks from RegEx literal
    let regex = regex.as_str()[regex.find('"').unwrap() + 1..regex.rfind('"').unwrap()].to_string();

    Ok((name, regex))
}

#[proc_macro]
pub fn parse_regex(tokens: TokenStream) -> TokenStream {
    // Parse the tokens into a name and a RegEx literal
    // TODO: CompileError report
    let (name, regex) = parse_token_stream(tokens).unwrap();
    // format!(r###"println!("{{}}", r##"{:?}"##)"###, (name, regex)).parse().unwrap()

    // Parse the RegEx into actual code
    let code = parse_regex_string(&name, &regex);

    // Parse the code into Rust tokens
    return code.parse().unwrap()
}
