#![allow(dead_code)]
use std::fmt::Display;

use regex_syntax::ast::*;
use regex_syntax::hir::HirKind;
use regex_syntax::hir::translate;
use regex_syntax::hir::Class as HirClass;

use crate::types::CompileError;

pub trait IR: Sized {
    fn parse(ast: &regex_syntax::ast::Ast) -> Result<Self, CompileError>;
    fn generate_impl(self) -> Vec<RegExpImplementation>;
}

pub struct RegExpImplementation {
    pub ranges: Option<Vec<(char, char)>>,
    pub is_match: String,
    pub is_match_at: String,
    pub matches: Option<String>,
    pub find_match: String,
    pub find_match_at: String,
    pub min_len: usize,
    pub exp: Ast,
    pub name: String,
    pub sub_exp: Vec<RegExpImplementation>
}

impl Display for RegExpImplementation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

        let char_ranges = if let Some(r) = &self.ranges {
            format!("\nconst CHAR_RANGES: {};\n",
                    character_ranges_to_array(r))
        } else {
            String::new()
        };

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

// need to keep track of the expression
// need to only generate module for the root expression

pub struct RegExNode(Ast, RegExp);

#[derive(Debug)]
pub enum RegExp {
    Char(Character)
}

impl IR for RegExNode {
    fn parse(ast: &regex_syntax::ast::Ast) -> Result<Self, CompileError> {
        match ast {
            Ast::Literal(_) | Ast::Class(_) =>
                Ok(RegExNode(ast.clone(), RegExp::Char(Character::parse(ast)?))),
            _ =>  todo!("todo RegExNode")
        }
    }

    fn generate_impl(self) -> Vec<RegExpImplementation> {
        match self.1 {
            RegExp::Char(x) => x.generate_impl(),
        }
    }
}

/// Any possible single character as defined by the RegEx crate "regex" but streamlined for code generation.
#[derive(Debug)]
pub enum Character {
    Class(CharacterClass),
    Char(CharacterSingle)
}

impl IR for Character {
    fn parse(ast: &Ast) -> Result<Self, CompileError> {
        let single = CharacterSingle::parse(ast).map(Character::Char);

        if single.is_ok() {
            return single
        } else {
            let class = CharacterClass::parse(ast).map(Character::Class);
            if class.is_ok() {
                return class
            } else {
                return single
            }
        }
    }

    fn generate_impl(self) -> Vec<RegExpImplementation> {
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
            &Ast::Literal(Literal {c, ..}) => Ok(Self(ast.clone(), c)),
            _ => Err(CompileError::UnexpectedToken(0, 0))
        }
    }

    fn generate_impl(self) -> Vec<RegExpImplementation> {
        let struct_name = regex_name!(self.0.to_string());

        let find_match_at = format!(
     r#"if input.chars().nth(offset) == Some('{}') {{{{
            Some((offset, offset))
        }}}} else {{{{
            None
        }}}}"#, self.1.escape_unicode()
                    .to_string()
                    .replace("{", "{{")
                    .replace("}", "}}"));

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

        let is_match_at = format!(
            r#"input.chars().nth(offset) == Some('{}')"#,
            self.1.escape_unicode()
                    .to_string()
                    .replace("{", "{{")
                    .replace("}", "}}"));

        let is_match = format!(
            r#"input.contains('{}')"#,
            self.1.escape_unicode()
                    .to_string()
                    .replace("{", "{{")
                    .replace("}", "}}"));

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

// For later
/// Find the character range for a singular unicode category in the form of an AST singleton
fn unicode_ranges(ast: &Ast) -> Vec<(char, char)> {
    let ir = translate::Translator::new()
        .translate(&ast.to_string(), ast)
        .map_err(|_| todo!("get unicode range error"))
        .unwrap();

    match ir.into_kind() {
        HirKind::Class(HirClass::Unicode(x)) => {
            x.ranges().iter().map(|x| (x.start(), x.end())).collect()
        }
        _ => panic!("Unhandled HIR struct")
    }
}

fn character_ranges_to_array(ranges: &Vec<(char, char)>) -> String {
    let mut list = ranges.iter()
        .map(|range|
             format!(
                 r"('{}', '{}')",
                 range.0.escape_unicode()
                    .to_string()
                    .replace("{", "{{")
                    .replace("}", "}}"),
                 range.1.escape_unicode()
                    .to_string()
                    .replace("{", "{{")
                    .replace("}", "}}")))
        .collect::<Vec<_>>()
        .join(", ");

    list.insert(0, '[');
    list.push(']');

    list.insert_str(0, &format!("[(char, char); {}] = ", ranges.len()));

    list
}

impl IR for CharacterClass {
    fn parse(ast: &Ast) -> Result<Self, CompileError> {
        match ast {
            Ast::Class(_) => {
                let ranges = unicode_ranges(&ast);
                Ok(Self(ast.clone(), ranges))
            },
            _ => Err(CompileError::UnexpectedToken(0, 0))
        }
    }

    fn generate_impl(self) -> Vec<RegExpImplementation> {
        let struct_name = regex_name!(self.0.to_string());

        let find_match_at =
           "if let Some(c) = input.chars().nth(offset) {{{{
            if Self::CHAR_RANGES.into_iter().find(|&(l, r)| l <= c && c <= r).is_some() {{{{
                return Some((offset, offset))
            }}}}
        }}}}
        None".into();

        let find_match =
       "let mut i = 0;
        for c in input.chars() {{{{
            if Self::CHAR_RANGES.into_iter().find(|&(l, r)| l <= c && c <= r).is_some() {{{{
                return Some((i, i))
            }}}}
            i += 1;
        }}}}
        None".into();


        let is_match_at = format!(
        "if let Some(c) = input.chars().nth(offset) {{{{
            Self::CHAR_RANGES.into_iter().find(|&(l, r)| l <= c && c <= r).is_some()
        }}}} else {{{{
            false
        }}}}");

        let is_match = "input.chars().into_iter().find(|&c| Self::CHAR_RANGES.into_iter().find(|&(l, r)| l <= c && c <= r).is_some()).is_some()".into();

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
