#![allow(dead_code)]
use std::fmt::Display;

use regex_syntax::ast::*;
use regex_syntax::hir::HirKind;
use regex_syntax::hir::translate;
use regex_syntax::hir::Class as HirClass;

use crate::types::CompileError;

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

    fn generate_impl(&self) -> Vec<RegExpImplementation> {
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

    fn generate_impl(&self) -> Vec<RegExpImplementation> {
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
            exp: self.0.to_owned(),
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

    fn generate_impl(&self) -> Vec<RegExpImplementation> {
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
            ranges: Some(self.1.to_owned()),
            find_match,
            find_match_at,
            min_len: 1,
            exp: self.0.to_owned(),
            name: struct_name,
            sub_exp: Vec::new(),
        }]
    }
}


#[derive(Debug)]
pub struct Concatination(Ast, Vec<RegExNode>);

impl IR for Concatination {
    fn parse(ast: &regex_syntax::ast::Ast) -> Result<Self, CompileError> {
        match ast {
            Ast::Concat(x) => {
                // make a vector with the length of the old one
                let cat = Vec::with_capacity(x.asts.len());
                // fold all of the asts into an array of parsed IR's
                let cat = x.asts.iter().try_fold(cat, |mut acc, ast| {
                    acc.push(RegExNode::parse(ast)?);
                    Ok(acc)
                })?;

                // return values
                Ok(Self(ast.clone(), cat))
            }
            // TODO: Improve error and location
            _ => Err(CompileError::UnexpectedToken(0, 0))
        }
    }

    fn generate_impl(&self) -> Vec<RegExpImplementation> {
        // Generate the underlying implementations
        let impls: Vec<_> = self.1.iter().map(|x| x.generate_impl()).collect();
        let struct_name = regex_name!(self.0.to_string());

        // Implementation of `find_match_at` for finding a concatination
        // Has a list of references to other functions that represent
        // the subexpressions and uses them to match the string
        // progressively. This is done by using the end position of
        // the last subexpression and adding one. The map at the end
        // is to remove the extra + 1 that will come from a successful
        // find.
        let find_match_at = format!(r"
            const {}: [{}; {}] = [{}];

            {}.into_iter().try_fold(offset, |acc, f| Some(f(input, acc)?.1 + 1)).map(|x| (offset, x - 1))",
            SUB_EXPR_LIST_NAME,
            FIND_MATCH_AT_TYPE_STRING,
            impls.len(),
            impls.iter().map(|x| {
                let mut access = x.last().unwrap().name.clone();
                access.push_str("::find_match_at");
                access
            }).collect::<Vec<_>>().join(", "),
            SUB_EXPR_LIST_NAME);


        // Implementation of `find_match` for finding a concatination
        // Loops through all of the indexes of the strnig and applies
        // `find_match_at`
        let find_match = format!(r"
            let len = input.chars().count();

            for i in 0..len {{{{
                let result = Self::find_match_at(input, i);
                if result.is_some() {{{{
                    return result
                }}}}
            }}}}

            None");

        // Implementation of `is_match_at` for finding a concatination
        // Calls on `find_match_at` on the offset and input to check if
        // the match exists
        let is_match_at = format!("Self::find_match_at(input, offset).is_some()");

        // Implementation of `is_match` for finding a concatination
        // Calls on `find_match` on the offset and input to check if
        // the match exists
        let is_match = format!("Self::find_match(input).is_some()");

        // Put everything together and return
        vec![RegExpImplementation {
            is_match,
            is_match_at,
            matches: None,
            ranges: None,
            find_match,
            find_match_at,
            min_len: impls.iter().fold(0, |acc, x| acc + x.last().unwrap().min_len),
            exp: self.0.to_owned(),
            name: struct_name,
            sub_exp: Vec::new(),
        }]
    }
}


#[derive(Debug)]
pub struct Alternation(Ast, Vec<RegExNode>);

impl IR for Alternation {
    fn parse(ast: &regex_syntax::ast::Ast) -> Result<Self, CompileError> {
        match ast {
            Ast::Alternation(x) => {
                // make a vector with the length of the old one
                let alt = Vec::with_capacity(x.asts.len());
                // fold all of the asts into an array of parsed IR's
                let alt = x.asts.iter().try_fold(alt, |mut acc, ast| {
                    acc.push(RegExNode::parse(ast)?);
                    Ok(acc)
                })?;

                // return values
                Ok(Self(ast.clone(), alt))
            }
            // TODO: Improve error and location
            _ => Err(CompileError::UnexpectedToken(0, 0))
        }
    }

    fn generate_impl(&self) -> Vec<RegExpImplementation> {
        // Generate the underlying implementations
        let impls: Vec<_> = self.1.iter().map(|x| x.generate_impl()).collect();
        let struct_name = regex_name!(self.0.to_string());

        // Implementation of `find_match_at` for finding a concatination
        // Has a list of references to other functions that represent
        // the subexpressions and uses them to find at least one match.
        let find_match_at = format!(r"
            const {}: [{}; {}] = [{}];

            {}.into_iter().find_map(|f| f(input, offset))",
            SUB_EXPR_LIST_NAME,
            FIND_MATCH_AT_TYPE_STRING,
            impls.len(),
            impls.iter().map(|x| {
                let mut access = x.last().unwrap().name.clone();
                access.push_str("::find_match_at");
                access
            }).collect::<Vec<_>>().join(", "),
            SUB_EXPR_LIST_NAME);


        // Implementation of `find_match` for finding a concatination
        // Loops through all of the indexes of the strnig and applies
        // `find_match_at`
        let find_match = format!(r"
            let len = input.chars().count();

            for i in 0..len {{{{
                let result = Self::find_match_at(input, i);
                if result.is_some() {{{{
                    return result
                }}}}
            }}}}

            None");

        // Implementation of `is_match_at` for finding a concatination
        // Calls on `find_match_at` on the offset and input to check if
        // the match exists
        let is_match_at = format!("Self::find_match_at(input, offset).is_some()");

        // Implementation of `is_match` for finding a concatination
        // Calls on `find_match` on the offset and input to check if
        // the match exists
        let is_match = format!("Self::find_match(input).is_some()");

        // Put everything together and return
        vec![RegExpImplementation {
            matches: None,
            ranges: None,
            is_match,
            is_match_at,
            find_match,
            find_match_at,
            min_len: impls.iter().map(|x| x.last().unwrap().min_len).min().unwrap(),
            exp: self.0.to_owned(),
            name: struct_name,
            sub_exp: Vec::new(),
        }]
    }
}
