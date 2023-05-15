use regex::internal::Inst;

use crate::ir::functions::{ProgramImplementation, self, time_name};

pub fn parse(instructions: &[Inst]) -> ProgramImplementation {
    let mut impls = Vec::with_capacity(instructions.len());

    // Macro cause Char, Ranges, and Bytes all have .goto but no shared trait for it
    macro_rules! simple_instruction_parsing {
        ($map:ident, $i:ident, $inst:ident, $x:ident) => {
            {
                let code = crate::ir::functions::instruction_code($inst);
                let code = format!("{}\n{}\n{}\nreturn Self::{}({}, {})",
                    crate::ir::functions::INNER_INDEX_INIT,
                    code,
                    crate::ir::functions::INNER_INDEX_END,
                    format!("F{}", $x.goto),
                    crate::ir::functions::INPUT_PARAM_NAME,
                    crate::ir::functions::INDEX_PARAM_NAME);

                let name = format!("F{}", $i);

                $map.push(crate::ir::functions::ProgramImplementation {
                    name,
                    body: code,
                    children: vec![]
                });
            }
        };
    }

    // use the goto value as an id for specific functions,
    // single character matches will be their own functions
    // this way when backtracking, a goto will be considered a call
    // to a function, which can then be found through the above map
    for (i, inst) in instructions.iter().enumerate() {
        match inst {
            Inst::Char(x) => simple_instruction_parsing!(impls, i, inst, x),
            Inst::Ranges(x) => simple_instruction_parsing!(impls, i, inst, x),
            Inst::Bytes(x) => simple_instruction_parsing!(impls, i, inst, x),

            Inst::Split(x) => {
                let code = format!("let mut index1 = *{0};\nlet mut index2 = *{0};\nif Self::F{2}({1}, &mut index1) {{\n    *{0} = index1;\n    return true\n}}\nelse if Self::F{3}({1}, &mut index2) {{\n    *{0} = index2;\n    return true\n}}\nelse {{\n    return false\n}}",
                    functions::INDEX_PARAM_NAME,
                    functions::INPUT_PARAM_NAME,
                    x.goto1,
                    x.goto2);

                let name = format!("F{}", i);

                impls.push(ProgramImplementation { body: code, name, children: vec![] });
            },

            // Utility: Ignore
            // Copy-Paste cause ICBA
            Inst::EmptyLook(x) => {
                let code = format!("return Self::{}({}, {})",
                    format!("F{}", x.goto),
                    functions::INPUT_PARAM_NAME,
                    functions::INDEX_PARAM_NAME);

                let name = format!("F{}", i);

                impls.push(functions::ProgramImplementation {
                    name,
                    body: code,
                    children: vec![]
                });
            },
            Inst::Save(x) => {
                let code = format!("return Self::{}({}, {})",
                    format!("F{}", x.goto),
                    functions::INPUT_PARAM_NAME,
                    functions::INDEX_PARAM_NAME);

                let name = format!("F{}", i);

                impls.push(functions::ProgramImplementation {
                    name,
                    body: code,
                    children: vec![]
                });
            },

            // Utility, but use as an end to the parsing
            Inst::Match(_) => {
                // always return true, since if its made its way here everything else is fulfilled
                let code = String::from("return true");

                let name = format!("F{}", i);

                impls.push(functions::ProgramImplementation {
                    name,
                    body: code,
                    children: vec![]
                });
            },
        }
    }

    let name = time_name!('F');

    let body = format!("return Self::{}({}, {})", 
        impls.first().unwrap().name,
        functions::INPUT_PARAM_NAME,
        functions::INDEX_PARAM_NAME);

    ProgramImplementation {
        name, body,
        children: impls
    }
}
