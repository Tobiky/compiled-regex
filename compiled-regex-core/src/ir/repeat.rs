#![allow(dead_code)]
use regex_syntax::ast::{Ast, Repetition as AstRepetition, RepetitionOp, RepetitionRange, RepetitionKind};

use crate::types::CompileError;

use super::{RegExNode, IR, RegExpImplementation};

#[derive(Debug)]
pub enum Repetition {
    ZeroMore(ZeroMore),
    OneMore(OneMore),
    ZeroOne(ZeroOne),
    Range(Range)
}

impl IR for Repetition {
    fn parse(ast: &Ast) -> Result<Self, crate::types::CompileError> {
        Ok(match ast {
            Ast::Repetition(AstRepetition { op: RepetitionOp { kind: RepetitionKind::ZeroOrMore, .. }, .. }) =>
                Repetition::ZeroMore(ZeroMore::parse(ast)?),
            Ast::Repetition(AstRepetition { op: RepetitionOp { kind: RepetitionKind::OneOrMore, .. }, .. }) =>
                Repetition::OneMore(OneMore::parse(ast)?),
            Ast::Repetition(AstRepetition { op: RepetitionOp { kind: RepetitionKind::ZeroOrOne, .. }, .. }) =>
                Repetition::ZeroOne(ZeroOne::parse(ast)?),
            Ast::Repetition(AstRepetition { op: RepetitionOp { kind: RepetitionKind::Range(_), .. }, .. }) =>
                Repetition::Range(Range::parse(ast)?),
            _ => return Err(CompileError::UnexpectedToken(0, 0))
        })
    }

    fn generate_impl(&self) -> Vec<super::RegExpImplementation> {
        match self {
            Repetition::ZeroMore(x) => x.generate_impl(),
            Repetition::OneMore(x) => x.generate_impl(),
            Repetition::ZeroOne(x) => x.generate_impl(),
            Repetition::Range(x) => x.generate_impl(),
        }
    }
}

/// Original AST, Lazy, Inner Node
#[derive(Debug)]
pub struct ZeroMore(pub Ast, pub bool, pub Box<RegExNode>);

impl IR for ZeroMore {
    fn parse(ast: &regex_syntax::ast::Ast) -> Result<Self, crate::types::CompileError> {
        match ast {
            Ast::Repetition(x) if matches!(&x.op.kind, RepetitionKind::ZeroOrMore) =>
                Ok(ZeroMore(ast.clone(), !x.greedy, Box::new(RegExNode::parse(&x.ast)?))),
            _ => Err(CompileError::UnexpectedToken(0, 0))
        }
    }

    fn generate_impl(&self) -> Vec<super::RegExpImplementation> {
        let mut impls = self.2.generate_impl();
        let sub_expr_name = &impls.last().unwrap().name;

        // Generate the name of the RegEx
        let struct_name = regex_name!(self.0.to_string());

        // Implementation of `find_match_at` for single characters
        // Either the subexpression is matched and the offset is moved
        // along the input ("One") or it wasn't matched and the subexpression
        // is ignored ("Zero").
        let find_match_at = format!(
            "let mut i = offset + 1;
             let mut r = {}::find_match_at(input, i);
             while r.is_some() {{{{
                 i += 1;
                 r = {}::find_match_at(input, i);
             }}}}

             if i == offset + 1 {{{{
                 let default = offset.checked_sub(1).unwrap_or(0);
                 Some((default, default))
             }}}} else {{{{
                 Some((i, i))
             }}}}",
            sub_expr_name,
            sub_expr_name
        );


        // Implementation of `find_match` for single characters
        // Iterate string with `find_match_at`
        let find_match = format!(
            "let len = input.chars().count();

            for i in 0..len {{{{
                let result = Self::find_match_at(input, i);
                if result.is_some() {{{{
                    return result
                }}}}
            }}}}

            Some((0,0))"
        );

        // Implementation of `is_match_at` for single characters
        // Use `find_match_at` to see if there is a match at the offset
        let is_match_at = format!(
            "Self::find_match_at(input, offset).is_some()"
        );

        // Implementation of `is_match` for single characters
        // Use `find_match` to check if there is any match
        let is_match = format!(
            "Self::find_match(input).is_some()"
        );

        // Put everything together into the implementation and attach
        // it at the end of the sub expressions
        impls.push(RegExpImplementation {
            is_match,
            is_match_at,
            ranges: None,
            matches: None,
            find_match,
            find_match_at,
            min_len: 0,
            exp: self.0.to_owned(),
            name: struct_name,
            sub_exp: Vec::new(),
        });

        impls
    }
}

/// Original AST, Lazy, Inner Node
#[derive(Debug)]
pub struct OneMore(pub Ast, pub bool, pub Box<RegExNode>);

impl IR for OneMore {
    fn parse(ast: &regex_syntax::ast::Ast) -> Result<Self, crate::types::CompileError> {
        match ast {
            Ast::Repetition(x) if matches!(&x.op.kind, RepetitionKind::OneOrMore) =>
                Ok(OneMore(ast.clone(), !x.greedy, Box::new(RegExNode::parse(&x.ast)?))),
            _ => Err(CompileError::UnexpectedToken(0, 0))
        }
    }

    fn generate_impl(&self) -> Vec<super::RegExpImplementation> {
        todo!("OneMore generate_impl")
    }
}

/// Original AST, Inner Node
#[derive(Debug)]
pub struct ZeroOne(pub Ast, pub Box<RegExNode>);

impl IR for ZeroOne {
    fn parse(ast: &regex_syntax::ast::Ast) -> Result<Self, crate::types::CompileError> {
        match ast {
            Ast::Repetition(x) if matches!(&x.op.kind, RepetitionKind::ZeroOrOne) =>
                Ok(ZeroOne(ast.clone(), Box::new(RegExNode::parse(&x.ast)?))),
            _ => {
                println!("HERO");
                Err(CompileError::UnexpectedToken(0, 0))
            }
        }
    }

    fn generate_impl(&self) -> Vec<super::RegExpImplementation> {
        let mut impls = self.1.generate_impl();

        // Generate the name of the RegEx
        let struct_name = regex_name!(self.0.to_string());

        // Implementation of `find_match_at` for single characters
        // Either the subexpression is matched and the offset is moved
        // along the input ("One") or it wasn't matched and the subexpression
        // is ignored ("Zero").
        // FIXME: (offset, offset) is not true, needs to be the previous state
        // position but only for iterations. Maybe min(offset - 1, 0)?
        let find_match_at = format!(
            "let default = offset.checked_sub(1).unwrap_or(0);
             let default = (default, default);
             Some({}::find_match_at(input,offset).unwrap_or(default))",
            impls.last().unwrap().name
        );

        // Implementation of `find_match` for single characters
        // Iterate string with `find_match_at`
        let find_match = format!(
            "let len = input.chars().count();

            for i in 0..len {{{{
                let result = Self::find_match_at(input, i);
                if result.is_some() {{{{
                    return result
                }}}}
            }}}}

            Some((0,0))"
        );

        // Implementation of `is_match_at` for single characters
        // Use `find_match_at` to see if there is a match at the offset
        let is_match_at = format!(
            "Self::find_match_at(input, offset).is_some()"
        );

        // Implementation of `is_match` for single characters
        // Use `find_match` to check if there is any match
        let is_match = format!(
            "Self::find_match(input).is_some()"
        );

        // Put everything together into the implementation and attach
        // it at the end of the sub expressions
        impls.push(RegExpImplementation {
            is_match,
            is_match_at,
            ranges: None,
            matches: None,
            find_match,
            find_match_at,
            min_len: 0,
            exp: self.0.to_owned(),
            name: struct_name,
            sub_exp: Vec::new(),
        });

        impls
    }
}

/// Original AST, Lazy, At Least, Start, End, Inner Node
#[derive(Debug)]
pub struct Range(pub Ast, pub bool, pub bool, pub u32, pub Option<u32>, pub Box<RegExNode>);

impl Range {
    fn generate_at_least_impl(&self) -> Vec<RegExpImplementation> {
        todo!()
    }

    fn generate_at_least_lazy_impl(&self) -> Vec<RegExpImplementation> {
        todo!()
    }
}

impl IR for Range{
    fn parse(ast: &regex_syntax::ast::Ast) -> Result<Self, crate::types::CompileError> {
        match ast {
            Ast::Repetition(x) => match &x.op.kind {
                RepetitionKind::Range(m) => {
                    // Turn the enum information into a flag and a range with optional end
                    let (f, a, b) = match m {
                        &RepetitionRange::Exactly(s) => (false, s, None),
                        &RepetitionRange::AtLeast(s) => (true, s, None),
                        &RepetitionRange::Bounded(s, e) => (false, s, Some(e))
                    };
                    Ok(Range(ast.clone(), !x.greedy, f, a, b, Box::new(RegExNode::parse(&x.ast)?)))
                },
                _ => Err(CompileError::UnexpectedToken(0, 0))
            },
            _ => Err(CompileError::UnexpectedToken(0, 0))
        }
    }

    fn generate_impl(&self) -> Vec<super::RegExpImplementation> {
        todo!("Range generate_impl")
    }
}
