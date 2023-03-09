use super::{RegExNode, RegExpImplementation, FIND_MATCH_AT_TYPE_STRING, IR, SUB_EXPR_LIST_NAME};
use crate::types::CompileError;
use regex_syntax::ast::Ast;

#[derive(Debug)]
pub struct Concatination(pub Ast, pub Vec<RegExNode>);

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
            _ => Err(CompileError::UnexpectedToken(0, 0)),
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
        let find_match_at = format!(
            r"
            const {}: [{}; {}] = [{}];

            {}.into_iter().try_fold(offset, |acc, f| Some(f(input, acc)?.1 + 1)).map(|x| (offset, x - 1))",
            SUB_EXPR_LIST_NAME,
            FIND_MATCH_AT_TYPE_STRING,
            impls.len(),
            impls
                .iter()
                .map(|x| {
                    let mut access = x.last().unwrap().name.clone();
                    access.push_str("::find_match_at");
                    access
                })
                .collect::<Vec<_>>()
                .join(", "),
            SUB_EXPR_LIST_NAME
        );

        // Implementation of `find_match` for finding a concatination
        // Loops through all of the indexes of the strnig and applies
        // `find_match_at`
        let find_match = format!(
            r"
            let len = input.chars().count();

            for i in 0..len {{{{
                let result = Self::find_match_at(input, i);
                if result.is_some() {{{{
                    return result
                }}}}
            }}}}

            None"
        );

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
            min_len: impls
                .iter()
                .fold(0, |acc, x| acc + x.last().unwrap().min_len),
            exp: self.0.to_owned(),
            name: struct_name,
            sub_exp: Vec::new(),
        }]
    }
}

#[derive(Debug)]
pub struct Alternation(pub Ast, pub Vec<RegExNode>);

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
            _ => Err(CompileError::UnexpectedToken(0, 0)),
        }
    }

    fn generate_impl(&self) -> Vec<RegExpImplementation> {
        // Generate the underlying implementations
        let impls: Vec<_> = self.1.iter().map(|x| x.generate_impl()).collect();
        let struct_name = regex_name!(self.0.to_string());

        // Implementation of `find_match_at` for finding a concatination
        // Has a list of references to other functions that represent
        // the subexpressions and uses them to find at least one match.
        let find_match_at = format!(
            r"
            const {}: [{}; {}] = [{}];

            {}.into_iter().find_map(|f| f(input, offset))",
            SUB_EXPR_LIST_NAME,
            FIND_MATCH_AT_TYPE_STRING,
            impls.len(),
            impls
                .iter()
                .map(|x| {
                    let mut access = x.last().unwrap().name.clone();
                    access.push_str("::find_match_at");
                    access
                })
                .collect::<Vec<_>>()
                .join(", "),
            SUB_EXPR_LIST_NAME
        );

        // Implementation of `find_match` for finding a concatination
        // Loops through all of the indexes of the strnig and applies
        // `find_match_at`
        let find_match = format!(
            r"
            let len = input.chars().count();

            for i in 0..len {{{{
                let result = Self::find_match_at(input, i);
                if result.is_some() {{{{
                    return result
                }}}}
            }}}}

            None"
        );

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
            min_len: impls
                .iter()
                .map(|x| x.last().unwrap().min_len)
                .min()
                .unwrap(),
            exp: self.0.to_owned(),
            name: struct_name,
            sub_exp: Vec::new(),
        }]
    }
}
