#![allow(dead_code)]
use std::fmt::Display;

use regex_syntax::ast::*;
use regex_syntax::hir::HirKind;
use regex_syntax::hir::translate;
use regex_syntax::hir::Class as HirClass;

use crate::types::CompileError;

// TODO: Set parse input and generate output to type parameters
/// General trait for Intermediate Representation
pub trait IR: Sized {
    /// Parse the RegEx AST into an IR that can be used to compile to Rust code
    fn parse(ast: &regex_syntax::ast::Ast) -> Result<Self, CompileError>;
    /// Generate the actual implementation that can be written as Rust code
    fn generate_impl(self) -> Vec<RegExpImplementation>;
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
pub struct RegExNode(Ast, RegExp);

/// The largest possible gathering of RegEx parts, for example
/// single character sequences or any character class
#[derive(Debug)]
pub enum RegExp {
    /// Single character match
    Char(Character)
}

impl IR for RegExNode {
    fn parse(ast: &regex_syntax::ast::Ast) -> Result<Self, CompileError> {
        match ast {
            // Both a literal and a class of characters fall under
            // "a single character"
            Ast::Literal(_) | Ast::Class(_) =>
                Ok(RegExNode(ast.clone(), RegExp::Char(Character::parse(ast)?))),
            _ =>  todo!("todo RegExNode")
        }
    }

    fn generate_impl(self) -> Vec<RegExpImplementation> {
        // TODO: find way to use generate_impl on all without copy+paste
        match self.1 {
            // Single character implementation is just its own implementation
            RegExp::Char(x) => x.generate_impl(),
        }
    }
}

/// Any possible single character as defined by the RegEx crate "regex" but streamlined for code generation.
#[derive(Debug)]
pub enum Character {
    /// A Class or Union of characters, like '\w' or '[A-Z]'
    Class(CharacterClass),
    /// A single simple character, like 'a'
    Char(CharacterSingle)
}

impl IR for Character {
    fn parse(ast: &Ast) -> Result<Self, CompileError> {
        // Try to parse a single character first
        let single = CharacterSingle::parse(ast).map(Character::Char);

        if single.is_ok() {
            // Single character parse was successful, return result
            return single
        } else {
            // If single character parse fails, attempt to parse a character class
            let class = CharacterClass::parse(ast).map(Character::Class);
            if class.is_ok() {
                return class
            } else {
                // If the character class parse also fails then return an error
                // TODO: More specific error; currently just returning the first parse error
                return single
            }
        }
    }

    fn generate_impl(self) -> Vec<RegExpImplementation> {
        // Just sending back the respective implementations of the enum
        match self {
            Character::Char(x) => x.generate_impl(),
            Character::Class(x) => x.generate_impl(),
        }
    }
}

/// Defines a character class by providing the character ranges for it.
#[derive(Debug)]
pub struct CharacterClass(Ast, Vec<(char, char)>);

/// A single regex character, matched verbatim.
#[derive(Debug)]
pub struct CharacterSingle(Ast, char);

impl IR for CharacterSingle {
    fn parse(ast: &Ast) -> Result<Self, CompileError> {
        match ast {
            // All literals can be parsed/interpreted as single characters
            &Ast::Literal(Literal {c, ..}) => Ok(Self(ast.clone(), c)),
            // TODO: Improve error and location
            _ => Err(CompileError::UnexpectedToken(0, 0))
        }
    }

    fn generate_impl(self) -> Vec<RegExpImplementation> {
        // Generate the name of the RegEx
        let struct_name = regex_name!(self.0.to_string());

        // Implementation of `find_match_at` for single characters
        // Check if the character at the given offset is the same
        // as the reference character, if so return offset as range.
        let find_match_at = format!(
     r#"if input.chars().nth(offset) == Some('{}') {{{{
            Some((offset, offset))
        }}}} else {{{{
            None
        }}}}"#, self.1.escape_unicode()
                    .to_string()
                    .replace("{", "{{")
                    .replace("}", "}}"));

        // Implementation of `find_match` for single characters
        // Searches the entire string for the reference character
        // and returns the character index if it was found.
        let find_match = format!(
     r#"let mut i = 0;
        for c in input.chars() {{{{
            if c == '{}' {{{{
                return Some((i, i))
            }}}}
            i += 1;
        }}}}
        None"#,
            self.1.escape_unicode()
                    .to_string()
                    .replace("{", "{{")
                    .replace("}", "}}"));

        // Implementation of `is_match_at` for single characters
        // Checks if the character at the given offset matches the
        // reference character.
        let is_match_at = format!(
            r#"input.chars().nth(offset) == Some('{}')"#,
            self.1.escape_unicode()
                    .to_string()
                    .replace("{", "{{")
                    .replace("}", "}}"));

        // Implementation of `is_match` for single characters
        // Search the entire string for the reference character,
        // return a boolean for if it exists.
        let is_match = format!(
            r#"input.contains('{}')"#,
            self.1.escape_unicode()
                    .to_string()
                    .replace("{", "{{")
                    .replace("}", "}}"));

        // Put everything together into the implementation
        vec![RegExpImplementation {
            is_match,
            is_match_at,
            ranges: None,
            matches: None,
            find_match,
            find_match_at,
            min_len: 1,
            exp: self.0,
            name: struct_name,
            sub_exp: Vec::new(),
        }]
    }
}

// Very hacky
/// Find the character range for a singular unicode category in the form of an AST singleton
fn unicode_ranges(ast: &Ast) -> Vec<(char, char)> {
    // Translate the AST to a HIR
    let ir = translate::Translator::new()
        .translate(&ast.to_string(), ast)
        .map_err(|_| todo!("get unicode range error"))
        .unwrap();

    // To get the underlying unicode classification
    match ir.into_kind() {
        HirKind::Class(HirClass::Unicode(x)) => {
            // Get all ranges of unicode values and map them to tuples
            x.ranges().iter().map(|x| (x.start(), x.end())).collect()
        }
        // Some HIR that can't be used
        _ => panic!("Unhandled HIR struct")
    }
}

fn character_ranges_to_array(ranges: &Vec<(char, char)>) -> String {
    // Turn a vector of tuples into a string representing an array of tuples
    let mut list = ranges.iter()
        .map(|range|
             format!(
                 r"('{}', '{}')",
                 // Get the unicode literal but with extra braces
                 // to avoid formatting issues
                 range.0.escape_unicode()
                    .to_string()
                    .replace("{", "{{")
                    .replace("}", "}}"),
                 // Get the unicode literal but with extra braces
                 // to avoid formatting issues
                 range.1.escape_unicode()
                    .to_string()
                    .replace("{", "{{")
                    .replace("}", "}}")))
        .collect::<Vec<_>>()
        // Join the tuples together with a comma
        .join(", ");

    // Surround with [ ]
    list.insert(0, '[');
    list.push(']');

    // Rust type prefix
    list.insert_str(0, &format!("[(char, char); {}] = ", ranges.len()));

    list
}

impl IR for CharacterClass {
    fn parse(ast: &Ast) -> Result<Self, CompileError> {
        match ast {
            // Can only parse character classes
            Ast::Class(_) => {
                // Get the range of characters for this class
                let ranges = unicode_ranges(&ast);
                // Create the struct
                Ok(Self(ast.clone(), ranges))
            },
            // TODO: Improve error and location
            _ => Err(CompileError::UnexpectedToken(0, 0))
        }
    }

    fn generate_impl(self) -> Vec<RegExpImplementation> {
        // Generate name for RegEx
        let struct_name = regex_name!(self.0.to_string());

        // Implementation of `find_match_at` for single characters
        // Check that the character at the given offset is within
        // any of the character ranges.
        let find_match_at =
       "if let Some(c) = input.chars().nth(offset) {{{{
            if Self::CHAR_RANGES.into_iter().find(|&(l, r)| l <= c && c <= r).is_some() {{{{
                return Some((offset, offset))
            }}}}
        }}}}
        None".into();

        // Implementation of `find_match_at` for single characters
        // Search the string for any character that exists within
        // the character ranges.
        let find_match =
       "let mut i = 0;
        for c in input.chars() {{{{
            if Self::CHAR_RANGES.into_iter().find(|&(l, r)| l <= c && c <= r).is_some() {{{{
                return Some((i, i))
            }}}}
            i += 1;
        }}}}
        None".into();

        // Implementation of `find_match_at` for single characters
        // Check that the character at the given offset is within
        // any of the character ranges.
        let is_match_at = format!(
        "if let Some(c) = input.chars().nth(offset) {{{{
            Self::CHAR_RANGES.into_iter().find(|&(l, r)| l <= c && c <= r).is_some()
        }}}} else {{{{
            false
        }}}}");

        // Implementation of `find_match_at` for single characters
        // Search the string for any character that exists within
        // the character ranges.
        let is_match = "input.chars().into_iter().find(|&c| Self::CHAR_RANGES.into_iter().find(|&(l, r)| l <= c && c <= r).is_some()).is_some()".into();

        // Put everything together and return
        vec![RegExpImplementation {
            is_match,
            is_match_at,
            matches: None,
            ranges: Some(self.1),
            find_match,
            find_match_at,
            min_len: 1,
            exp: self.0,
            name: struct_name,
            sub_exp: Vec::new(),
        }]
    }
}
