use super::{IR, RegExpImplementation};
use regex_syntax::{ast::{Ast, Literal}, hir::{HirKind, translate, Class as HirClass}};
use crate::types::CompileError;

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
pub struct CharacterClass(pub Ast, pub Vec<(char, char)>);

/// A single regex character, matched verbatim.
#[derive(Debug)]
pub struct CharacterSingle(pub Ast, pub char);

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

pub fn character_ranges_to_array(ranges: &Vec<(char, char)>) -> String {
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
