#![allow(dead_code)]
use regex::internal::Compiler;
use regex::internal::Inst;
use regex_syntax::Parser;

mod ir;
mod parser;
pub mod types;
pub use ir::functions::CHAR_GET_FUNC;

use parser::parse;

pub fn parse_regex(
    input: &str,
) -> Result<ir::functions::ProgramImplementation, types::CompileError> {
    let hir = Parser::new()
        .parse(input)
        .map_err(types::CompileError::RegexSyntaxError)?;

    let program = Compiler::new()
        .compile(std::slice::from_ref(&hir))
        .map_err(types::CompileError::RegexError)?;

    #[cfg(debug_assertions)]
    println!("Program \"{}\":\n{:?}", input, program);

    parse_program(&program.insts)
}

fn parse_program<'lt>(
    instructions: &'lt [Inst],
) -> Result<ir::functions::ProgramImplementation, types::CompileError> {
    // let program = ir::sections::Program::try_parse(instructions)?;

    // ir::functions::ProgramImplementation::try_parse(&program)
    
    Ok(parse(instructions))
}
