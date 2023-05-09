use regex::Error as RegexError;
use regex_syntax::Error as RegexSyntaxError;

#[derive(Debug)]
pub enum CompileError {
    UnexpectedToken(usize, usize),
    RegexSyntaxError(RegexSyntaxError),
    RegexError(RegexError),
    TODO,
}

pub type Result<T> = core::result::Result<T, CompileError>;
