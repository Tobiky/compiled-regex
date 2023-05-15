#[allow(unused_imports)]
use std::{fmt::Display, time::{SystemTime, UNIX_EPOCH}, char::EscapeUnicode};

use regex::internal::Inst;

use sha2::{Digest, Sha256};

use const_format::formatcp;

use super::sections::Program;

use crate::types::Result;

pub struct ProgramImplementation {
    pub body: String,
    pub name: String,
    pub children: Vec<ProgramImplementation>,
}

pub const INPUT_PARAM_NAME: &'static str = "input";
pub const INPUT_PARAM_TYPE: &'static str = "&str";
pub const INDEX_PARAM_NAME: &'static str = "index";
pub const INDEX_PARAM_TYPE_INNER: &'static str = "usize";
pub const INDEX_PARAM_TYPE: &'static str = formatcp!("&mut {INDEX_PARAM_TYPE_INNER}");
pub const CHAR_GET_FUNC_NAME: &'static str = "__get_char";
// public since its faster than attaching it as a child to the root ProgramImplementation
pub const CHAR_GET_FUNC: &'static str =  formatcp!(
"fn {CHAR_GET_FUNC_NAME}({INPUT_PARAM_NAME}: {INPUT_PARAM_TYPE}, {INDEX_PARAM_NAME}: {INDEX_PARAM_TYPE}) -> Option<char> {{
    {INPUT_PARAM_NAME}[*{INDEX_PARAM_NAME}..].chars().next()
}}\n");
pub const INNER_INDEX_NAME: &'static str = "inner_index";
pub const INNER_INDEX_INIT: &'static str = formatcp!("let mut {INNER_INDEX_NAME}: {INDEX_PARAM_TYPE_INNER} = *{INDEX_PARAM_NAME};");
pub const INNER_INDEX_END: &'static str = formatcp!("*{INDEX_PARAM_NAME} = {INNER_INDEX_NAME};");

static mut PROG_COUNTER: usize = 0;

macro_rules! hash_name {
    ($prefix:literal, $($x:expr),* ) => {
        {
            use sha2::{Digest, Sha256};

            let mut hasher = Sha256::new();

            $(
                hasher.update($x);
            )*

            hash_name!(hasher, $prefix)
        }
    };
    ($hasher:ident, $prefix:literal) => {
        {
            let result = $hasher.finalize();
            let mut name = hex::encode(result);
            name.insert(0, $prefix);
            name.insert_str(0, "__");
            name
        }
    };
    ($hasher:ident) => {
        hash_name!($hasher, 'F')
    };
}

macro_rules! time_name {
    () => {
        time_name!('F')
    };
    ($prefix:literal) => {
        format!("__{}{}", $prefix, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
    };
}

pub(crate) use hash_name;
pub(crate) use time_name;

impl<'a> ProgramImplementation {
    pub fn empty() -> Self {
        ProgramImplementation {
            name: String::new(),
            body: String::new(),
            children: vec![]
        }
    }

    pub(crate) fn try_parse(
        program: &Program,
    ) -> Result<ProgramImplementation> {
        match program {
            Program::Normal(inst) => try_parse_instructions(inst),
            Program::Loop(program) => try_parse_loop(program),
            Program::Choice(a, b) => try_parse_choice(a, b),
            Program::Linear(programs) => try_parse_linear(programs),
        }
    }
}

impl Display for ProgramImplementation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.children.iter().try_for_each(|x| write!(f, "{}",x))?;
        write!(f,
            /*\n#[inline(auto)]*/"\nfn {0}({INPUT_PARAM_NAME}: {INPUT_PARAM_TYPE}, {INDEX_PARAM_NAME}: {INDEX_PARAM_TYPE}) -> bool {{\n    {1}\n}}\n",
            self.name,
            self.body.replace("\n", "\n    "))
    }
}

fn character_range_to_literal(range: &[(char, char)], literal_name: &str) -> String {
    let ranges = range
        .iter()
        .map(|(start, end)|
            format!("('{}', '{}')",
                start.escape_unicode(),
                end.escape_unicode()))
        .collect::<Vec<_>>();

    format!("const {literal_name}: [(char, char); {}] = [{}];\n", ranges.len(), ranges.join(", "))
}

pub fn instruction_code(
    instruction: &Inst,
) -> String {
    match instruction {
        // Single Character match
        Inst::Char(x) => {
            format!("
// From: {instruction:?}
if Self::{CHAR_GET_FUNC_NAME}({INPUT_PARAM_NAME}, &mut {INNER_INDEX_NAME}) != Some('{0}') {{
    return false
}}
{INNER_INDEX_NAME} += {1};", x.c.escape_unicode(), x.c.len_utf8())
        },

        // Range of Characters match
        Inst::Ranges(x) => {
            let mut hasher = Sha256::new();

            x.ranges.iter().for_each(|&(start, end)| {
                hasher.update((start as u32).to_ne_bytes());
                hasher.update((end as u32).to_ne_bytes());
            });

            let range_name = hash_name!(hasher, 'R');

            format!("
// From: {instruction:?}
{0}
let {2} = Self::{CHAR_GET_FUNC_NAME}({INPUT_PARAM_NAME}, &mut {INNER_INDEX_NAME});
if {2}.is_some() {{
    let {2} = {2}.unwrap();
    if let Some((start, _)) = {1}.iter().find(|(start, end)| *start <= {2} && {2} <= *end) {{
        {INNER_INDEX_NAME} += start.len_utf8();
    }} else {{
        return false
    }}
}} else {{
    return false
}}
", character_range_to_literal(&x.ranges, &range_name), range_name, time_name!('V'))
        },

        // Range of Bytes
        // can be simplified if indexed byte is drawn out to a variable
        Inst::Bytes(x) => format!("
// From: {instruction:?}
if {}u8 <= {INPUT_PARAM_NAME}.as_bytes()[INNER_INDEX_NAME] && {INPUT_PARAM_NAME}.as_bytes()[INNER_INDEX_NAME] <= {}u8 {{
    return false
}}
{INDEX_PARAM_NAME} += 1;\n",
            x.start, x.end),
        // match is only used for regex sets
        Inst::Match(_) |
        // save is used for location saving
        Inst::Save(_) |
        // zero-width assertions, might not be necessary for our scope
        Inst::EmptyLook(_) => 
            String::new(),
        x => panic!("try_parse_instructions: {x:?} did not expect instruction type.")
    }
}


fn try_parse_instructions(
    instructions: &[Inst],
) -> Result<ProgramImplementation> {
    let prog_num = unsafe {
        PROG_COUNTER += 1;
        PROG_COUNTER
    };

    let mut parts = instructions.iter().map(instruction_code).collect::<Vec<_>>();

    let mut init = String::from(INNER_INDEX_INIT);
    init.insert_str(0, "    ");
    parts.insert(0, init);

    let mut end = String::from(INNER_INDEX_END);
    end.insert_str(0, "    ");
    parts.push(end);
    parts.push(String::from("    return true"));

    let mut body = parts.join("\n");

    body.insert_str(0, &format!("    // Index: {}\n", prog_num));

    let mut hasher = Sha256::new();

    hasher.update(&body);
    hasher.update(prog_num.to_ne_bytes());

    let name = hash_name!(hasher);

    Ok(ProgramImplementation {
        name,
        body,
        children: vec![],
    })
}

// FIXME: Does not care for greedyness
fn try_parse_loop(
    prog_loop: &Program,
) -> Result<ProgramImplementation> {
    let prog_num = unsafe {
        PROG_COUNTER += 1;
        PROG_COUNTER
    };

    let loop_body = ProgramImplementation::try_parse(prog_loop)?;

    // FIXME: not fully thought out, what to return?
    let mut body = format!("
    {INNER_INDEX_INIT}
    loop {{
        if !Self::{0}({INPUT_PARAM_NAME}, &mut {INNER_INDEX_NAME}) {{
            break;
        }}
        {INNER_INDEX_NAME} += 1;
    }}
    {INNER_INDEX_END}
    return true",
        loop_body.name
    );

    body.insert_str(0, &format!("    // Index: {}\n", prog_num));

    let mut hasher = Sha256::new();

    hasher.update(&body);
    hasher.update(prog_num.to_ne_bytes());

    let name = hash_name!(hasher);

    Ok(ProgramImplementation {
        name,
        body,
        children: vec![loop_body],
    })
}

fn try_parse_choice(
    prog_a: &Program,
    prog_b: &Program,
) -> Result<ProgramImplementation> {
    let prog_num = unsafe {
        PROG_COUNTER += 1;
        PROG_COUNTER
    };


    let prog_a = ProgramImplementation::try_parse(prog_a)?;
    let prog_b = ProgramImplementation::try_parse(prog_b)?;

    let mut body = format!("
    let mut prog_a_index = *{INDEX_PARAM_NAME};
    let mut prog_b_index = *{INDEX_PARAM_NAME};

    if Self::{0}({INPUT_PARAM_NAME}, &mut prog_a_index) {{
        *{INDEX_PARAM_NAME} = prog_a_index;
        return true
    }}
    else if Self::{1}({INPUT_PARAM_NAME}, &mut prog_b_index) {{
        *{INDEX_PARAM_NAME} = prog_b_index;
        return true
    }}
    else {{
        return false
    }}", prog_a.name, prog_b.name);

    body.insert_str(0, &format!("    // Index: {}\n", prog_num));

    let mut hasher = Sha256::new();

    hasher.update(body.as_bytes());
    hasher.update(prog_num.to_ne_bytes());

    let name = hash_name!(hasher);
    
    Ok(ProgramImplementation {
        name,
        body,
        children: vec![prog_a, prog_b],
    })
}

fn try_parse_linear(programs: &[Program]) -> Result<ProgramImplementation> { 
    let prog_num = unsafe {
        PROG_COUNTER += 1;
        PROG_COUNTER
    };

    let implementations = programs.iter().map(ProgramImplementation::try_parse).collect::<Result<Vec<_>>>()?;

    let mut body = format!("
    {INNER_INDEX_INIT}
    if {} {{
        {INNER_INDEX_END}
        return true
    }} else {{
        return false
    }}",
        implementations
            .iter()
            .map(|imple| format!("Self::{}({INPUT_PARAM_NAME}, &mut {INNER_INDEX_NAME})", imple.name))
            .collect::<Vec<_>>()
            .join(" && "));

    body.insert_str(0, &format!("    // Index: {}\n", prog_num));

    let mut hasher = Sha256::new();

    hasher.update(&body);
    hasher.update(prog_num.to_ne_bytes());

    let name = hash_name!(hasher);

    Ok(ProgramImplementation {
        name,
        body,
        children: implementations
    })
}

#[cfg(test)]
mod tests {
    use super::character_range_to_literal;

    #[test]
    fn character_range_literal_empty() {
        assert_eq!(
            &character_range_to_literal(&[], "LITERAL"),
            "const LITERAL: [char; 0] = [];"
        )
    }

    #[test]
    fn character_range_literal_two() {
        assert_eq!(
            &character_range_to_literal(&[('a', 'b'), ('e', 'ä')], "LITERAL"),
            "const LITERAL: [char; 2] = [('\\u{61}', '\\u{62}'), ('\\u{65}', '\\u{e4}')];"
        )
    }
}
