mod character;
mod composite;

use std::fmt::Display;

use regex_syntax::ast::*;

use crate::types::CompileError;

use character::{Character, character_ranges_to_array};
use composite::{Alternation, Concatination};

pub const FIND_MATCH_TYPE_STRING: &'static str =
    "fn (&str) -> Option<(usize, usize)>";
pub const FIND_MATCH_AT_TYPE_STRING: &'static str =
    "fn (&str, usize) -> Option<(usize, usize)>";
pub const MATCHPE_STRING: &'static str =
    "fn (&str) -> bool";
pub const MATCH_AT_TYPE_STRING: &'static str =
    "fn (&str, usize) -> bool";


pub const SUB_EXPR_LIST_NAME: &'static str =
    "SUB_EXPRS";


// TODO: Set parse input and generate output to type parameters
/// General trait for Intermediate Representation
pub trait IR: Sized {
    /// Parse the RegEx AST into an IR that can be used to compile to Rust code
    fn parse(ast: &regex_syntax::ast::Ast) -> Result<Self, CompileError>;
    /// Generate the actual implementation that can be written as Rust code
    fn generate_impl(&self) -> Vec<RegExpImplementation>;
}

/// Keeps track of how a RegEx is implemented (currently only through formated hardcoded strings).
/// Using `to_string` will generate the Rust code.
pub struct RegExpImplementation {
    /// Optional character ranges, saved as a constant array of character tuples `CHAR_RANGES`.
    pub ranges: Option<Vec<(char, char)>>,
    /// The implementation for the `is_match` method from `types::RegExp`
    pub is_match: String,
    /// The implementation for the `is_match_at` method from `types::RegExp`
    pub is_match_at: String,
    /// The optional implementation for the `matches` method from `types::RegExp`
    pub matches: Option<String>,
    /// The implementation for the `find_match` method from `types::RegExp`
    pub find_match: String,
    /// The implementation for the `find_match_at` method from `types::RegExp`
    pub find_match_at: String,
    /// The minimum length of the RegEx (currently not being generated)
    pub min_len: usize,
    /// The original AST from the parsed RegEx by [regex-syntax](https://docs.rs/regex-syntax/0.6.28/regex_syntax/index.html)
    pub exp: Ast,
    /// The name of the RegEx, usually the most significant half of the SHA256 hash of the
    /// expression string and prefixed with a double underscore.
    pub name: String,
    /// All subexpressions for this expression, they will be generated
    /// beforehand and can be refered by their `RegExpImplementation.name` field.
    pub sub_exp: Vec<RegExpImplementation>
}

impl Display for RegExpImplementation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Only overwrite the `matches` definition if a different implementation has been provided.
        let matches = if let Some(m) = &self.matches {
            format!(
 r#"#[inline(always)]
    fn matches(input: &str) -> Matches<Self> {{{{
        {}
    }}}}"#,
                m)
        } else {
            String::new()
        };

        // Only generate constant character range array if at least one
        // exists.
        let char_ranges = if let Some(r) = &self.ranges {
            format!("\nconst CHAR_RANGES: {};\n",
                    character_ranges_to_array(r))
        } else {
            String::new()
        };

        // Fill in the skeleton code with all implementations and values
        write!(f,
r#"pub struct {}();

#[allow(dead_code)]
impl {} {{{{
    {}
}}}}

#[allow(non_camel_case_types)]
impl compiled_regex::types::RegExp for {} {{{{
    const MIN_LEN: usize = {};

    #[inline(always)]
    fn find_match_at(input: &str, offset: usize) -> Option<(usize, usize)> {{{{
        {}
    }}}}

    #[inline(always)]
    fn find_match(input: &str) -> Option<(usize, usize)> {{{{
        {}
    }}}}

    #[inline(always)]
    fn is_match_at(input: &str, offset: usize) -> bool {{{{
        {}
    }}}}

    #[inline(always)]
    fn is_match(input: &str) -> bool {{{{
        {}
    }}}}{}
}}}}"#,
        self.name,
        self.name,
        char_ranges,
        self.name,
        self.min_len,
        self.find_match_at,
        self.find_match,
        self.is_match_at,
        self.is_match,
        matches)
    }
}

/// Keep track of a RegEx and its original AST
#[derive(Debug)]
pub struct RegExNode(Ast, RegExp);

/// The largest possible gathering of RegEx parts, for example
/// single character sequences or any character class
#[derive(Debug)]
pub enum RegExp {
    /// Single character match
    Char(Character),
    Concat(Concatination),
    Alt(Alternation),
}

impl IR for RegExNode {
    fn parse(ast: &regex_syntax::ast::Ast) -> Result<Self, CompileError> {
        Ok(RegExNode(ast.clone(), match ast {
            // Both a literal and a class of characters fall under
            // "a single character"
            Ast::Literal(_) | Ast::Class(_) =>
                RegExp::Char(Character::parse(ast)?),
            Ast::Concat(_) =>
                RegExp::Concat(Concatination::parse(ast)?),
            Ast::Alternation(_) =>
                RegExp::Alt(Alternation::parse(ast)?),
            _ => todo!("todo RegExNode")
        }))
    }

    fn generate_impl(&self) -> Vec<RegExpImplementation> {
        // TODO: find way to use generate_impl on all without copy+paste
        match &self.1 {
            // Single character implementation is just its own implementation
            RegExp::Char(x) => x.generate_impl(),
            // Subexpressions are applied first since they will be used in the
            // implementation of the concatination
            RegExp::Concat(x) => {
                let cat = Vec::with_capacity(x.1.len());
                let mut cat = x.1.iter().fold(cat, |mut acc, node| {
                    acc.append(&mut node.generate_impl());
                    acc
                });
                cat.append(&mut x.generate_impl());
                cat
            }
            // Subexpressions are applied first since they will be used in the
            // implementation of the alternation
            RegExp::Alt(x) => {
                let alt = Vec::with_capacity(x.1.len());
                let mut alt = x.1.iter().fold(alt, |mut acc, node| {
                    acc.append(&mut node.generate_impl());
                    acc
                });
                alt.append(&mut x.generate_impl());
                alt
            }
        }
    }
}
