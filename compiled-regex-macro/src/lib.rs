use compiled_regex_core::types::CompileError;
use proc_macro::TokenStream;
use proc_macro::{self, TokenTree};

use sha2::{Sha256, Digest};
use hex::encode;

use compiled_regex_core::{parse_regex as parse_regex_program, CHAR_GET_FUNC};

use litrs::StringLit;

fn parse_regex_string(
    export_name: &str,
    regex: &str,
) -> Result<String, CompileError> {
    let implementation = parse_regex_program(regex)?;

    // keeping hash name to improve upon algorithm later
    let mut hasher = Sha256::new();

    hasher.update(&implementation.name);
    hasher.update(&implementation.body);

    let result = hasher.finalize();
    let mut struct_name = encode(result);
    struct_name.insert(0, 'S');

    let struct_name = &struct_name[0..16];

    let code = format!("
struct __{struct_name}();
#[allow(unused_variables)]
#[allow(nonstandard_style)]
impl __{struct_name} {{
    {0}
    {1}

#[allow(dead_code)]
fn is_match(input: &str) -> bool {{
    Self::{2}(input, &mut 0)
}}
}}
type {export_name} = __{struct_name};",
    CHAR_GET_FUNC,
    implementation.to_string().replace("\n", "\n    "),
    implementation.name);

    Ok(code)
}

fn parse_token_stream(
    tokens: TokenStream,
) -> Result<(String, String), CompileError> {
    if tokens.is_empty() {
        // TODO: Should return error describing empty token set
        return Err(CompileError::TODO);
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
                return Err(CompileError::TODO);
            }
        }
        // TODO: Specifiy that identity or name is needed
        _ => return Err(CompileError::TODO),
    };

    // Make sure there is a delimiter inbetween
    // Can possibly skip this by just making sure that the the last
    // item is correct
    if !matches!(iter.next(), Some(TokenTree::Punct(_))) {
        // TODO: Specify that some punctuation is needed
        // currently can be any, not sure if it should be forced
        // to anything specific.
        return Err(CompileError::TODO);
    }

    // Get the regex string literal
    let regex = match iter.next() {
        Some(TokenTree::Literal(x)) => {
            if let Ok(s) = StringLit::try_from(x) {
                s.to_string()
            } else {
                // TODO: Specify illegal literal type usage
                return Err(CompileError::TODO);
            }
        }
        // TODO: Specifiy that str literal is needed
        _ => return Err(CompileError::TODO),
    };

    // Strip surrounding string marks from RegEx literal
    let regex = regex.as_str()
        [regex.find('"').unwrap() + 1..regex.rfind('"').unwrap()]
        .to_string();

    Ok((name, regex))
}

#[proc_macro]
pub fn parse_regex(tokens: TokenStream) -> TokenStream {
    // Parse the tokens into a name and a RegEx literal
    // TODO: CompileError report
    let (name, regex) = parse_token_stream(tokens).unwrap();
    // format!(r###"println!("{{}}", r##"{:?}"##)"###, (name, regex)).parse().unwrap()

    // Parse the RegEx into actual code
    let code = parse_regex_string(&name, &regex).unwrap();

    // Parse the code into Rust tokens
    return code.parse().unwrap();
}

#[proc_macro]
pub fn __parse_regex_generative_output(
    tokens: TokenStream,
) -> TokenStream {
    // Parse the tokens into a name and a RegEx literal
    // TODO: CompileError report
    let (name, regex) = parse_token_stream(tokens).unwrap();

    // Parse the RegEx into actual code
    let code = parse_regex_string(&name, &regex).unwrap();

    // Parse the code into Rust tokens
    return format!(r###"println!("{{}}", r##"{}"##)"###, code)
        .parse()
        .unwrap();
}
