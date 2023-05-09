use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use regex::internal::Inst;

use crate::types::Result;

type Ptr<T> = Rc<T>;

#[derive(Debug)]
pub(crate) enum Program<'a> {
    Normal(&'a [Inst]),
    Loop(Ptr<Program<'a>>),
    Choice(Ptr<Program<'a>>, Ptr<Program<'a>>),
    Linear(Vec<Program<'a>>),
}

impl<'a> Program<'a> {
    pub(crate) fn try_parse(
        instructions: &'a [Inst],
    ) -> Result<Program<'a>> {
        #[cfg(debug_assertions)]
        println!("Section Program Parsing:");

        let mut programs = HashMap::new();
        let program = try_parse_program(
            &mut programs,
            instructions,
            0,
            instructions.len(),
        );
        #[cfg(debug_assertions)]
        println!("\nSectioned Program:\n{:#?}", program);
        program
    }
}

fn get_program<'a>(
    programs: &mut HashMap<usize, Rc<Program<'a>>>,
    instructions: &'a [Inst],
    start: usize,
    end: usize,
) -> Result<Rc<Program<'a>>> {
    // Check if program already exists
    if let Some(program) = programs.get(&start) {
        Ok(program.clone())
    }
    // If not, parse the section, register it, and return it
    else {
        // Parse section specified by start and end
        let program =
            try_parse_program(programs, instructions, start, end)?;
        // Surround with pointer
        let program = Ptr::new(program);
        // Register to location
        _ = programs.insert(start, program.clone());
        // Return pointer
        Ok(program)
    }
}

fn try_parse_program<'a>(
    programs: &mut HashMap<usize, Rc<Program<'a>>>,
    instructions: &'a [Inst],
    start: usize,
    end: usize,
) -> Result<Program<'a>> {
    let mut sections = Vec::new();
    let mut i = start;
    let mut unused_insts_since = i;

    fn update_unused_since(unused: &mut usize, new: usize) {
        if unused == &0 {
            *unused = new;
        } else {
            *unused = usize::min(*unused, new);
        }
    }

    // Program "[abc]b|(t*)*":
    // 0000 Save(0) (start)
    // 0001 Split(2, 4)
    // 0002 'a'-'c'
    // 0003 'b' (goto: 9)     <- end of Alt LHS
    // 0004 Split(5, 9)       <- Start of (_)*
    // 0005 Save(2)
    // 0006 Split(7, 8)       <- Start of t*
    // 0007 't' (goto: 6)     <- End of t*
    // 0008 Save(3) (goto: 4) <- End of (_)*
    // 0009 Save(1)
    // 0010 Match(0)

    // Program "a{1,3}b":
    // 0000 Save(0) (start)
    // 0001 'a'
    // 0002 Split(3, 6)
    // 0003 'a'
    // 0004 Split(5, 6)
    // 0005 'a'
    // 0006 'b'
    // 0007 Save(1)
    // 0008 Match(0)
    while i < end {
        let inst = &instructions[i];
        println!("i={:04}: {:?}", i, inst);

        match inst {
            Inst::Split(split) => {
                // Check if there are any unused instructions that
                // need to be pushed in
                if unused_insts_since != i {
                    sections.push(Program::Normal(
                        &instructions[unused_insts_since..i],
                    ));
                }

                // Find out if there is a loop end to this split
                let loop_end = find_loop_body_end(instructions, i, i);

                // A loop exists; zero or more (*), one or more (+),
                // or custom quanity with no end ({1,})
                if let Some(end) = loop_end {
                    // Parse the section within
                    let program = try_parse_program(
                        programs,
                        instructions,
                        i + 1,
                        end,
                    )?;
                    // Surround it with a pointer
                    let program = Ptr::new(program);

                    // Set section to loop
                    let section = Program::Loop(program);
                    // Add section to list
                    sections.push(section);

                    // Update index
                    i = end;
                    unused_insts_since = i + 1;
                }
                // No loop; alternation (|), zero or one (?), or
                // custom quantity ({1,3})
                else {
                    // Parse the first section
                    let program_a = get_program(
                        programs,
                        instructions,
                        split.goto1,
                        split.goto2,
                    )?;

                    #[cfg(debug_assertions)]
                    {
                        println!(
                            "Program A ({}-{}):",
                            split.goto1, split.goto2
                        );
                        println!(
                            "{:?}",
                            &instructions[split.goto1..split.goto2]
                        );
                        println!("{:?}", &program_a);
                    }

                    // Find the joining fork for both programs
                    let program_b_end =
                        get_goto1(&instructions[split.goto2 - 1]) + 1;

                    // Parse the second section
                    let program_b = get_program(
                        programs,
                        instructions,
                        split.goto2,
                        program_b_end,
                    )?;

                    #[cfg(debug_assertions)]
                    {
                        println!(
                            "Program B ({}-{}):",
                            split.goto2, program_b_end
                        );
                        println!(
                            "{:?}",
                            &instructions[split.goto2..program_b_end]
                        );
                        println!("{:?}", &program_a);
                    }

                    // Set section to choice
                    let section =
                        Program::Choice(program_a, program_b);

                    // Add section to list
                    sections.push(section);

                    // Update index
                    i = program_b_end - 1;
                    unused_insts_since = i + 1;
                }
            }

            // Normal instructions, those that should just be put into
            // Program::Normal
            Inst::Char(_)
            | Inst::Bytes(_)
            | Inst::Ranges(_)
            | Inst::EmptyLook(_)
            | Inst::Save(_) => {
                update_unused_since(&mut unused_insts_since, i)
            }

            // The match instructions is only to tell the program
            // that a specific regex has been matched. Only relevant
            // for multiple regex programs
            Inst::Match(_) => break,
        }

        i += 1;
    }

    // Handle any potential unused instructions
    if unused_insts_since != i {
        sections.push(Program::Normal(
            &instructions[unused_insts_since..i],
        ));
    }

    // If there is only one section parsed, then just return that section
    if sections.len() == 1 {
        Ok(sections.pop().unwrap())
    }
    // Otherwise, return the sections as linear progression
    else {
        Ok(Program::Linear(sections))
    }
}

fn get_goto1(inst: &Inst) -> usize {
    match inst {
        Inst::Char(c) => c.goto,
        Inst::Ranges(r) => r.goto,
        Inst::Bytes(b) => b.goto,
        Inst::EmptyLook(e) => e.goto,
        // Split instructions contain branches which mean that
        // there is a need to handle further loops but still find
        // the backtrace to the header of the current loop.
        Inst::Split(s) => s.goto1,
        // All other instructions are utility instructions
        Inst::Save(x) => x.goto,
        // Instrucitons that do not lead anywhere
        _ => panic!("get_goto1: Assumed INST would have goto. {:?} does not have a goto.", inst),
    }
}

// FIXME: Does describe if loop is greedy or not
// May have to return a vector of references with the lifespan of the
// original vector.
/// Find the end of the loop by finding an instruction that points back
/// to the loop header
fn find_loop_body_end<'lt>(
    instructions: &'lt [Inst],
    start: usize,
    header: usize,
) -> Option<usize> {
    // first index is one of the indexes after the header, that is one
    // of the goto values a split instruction
    let mut inst_index = start;
    // collect all passed instructions to make sure that we do not enter
    // a loop
    let mut passed_inst = HashSet::new();

    // loop through all of the instructions and branches until it is
    // found
    loop {
        // make sure that the instruction has not already been passed
        if passed_inst.contains(&inst_index) {
            return None;
        }

        let next_index = match &instructions[inst_index] {
            // Normal match instructions, these will just be consecutive.
            // No need for fancy logic to deal with the indexes.
            Inst::Char(c) => c.goto,
            Inst::Ranges(r) => r.goto,
            Inst::Bytes(b) => b.goto,
            Inst::EmptyLook(e) => e.goto,
            // Split instructions contain branches which mean that
            // there is a need to handle further loops but still find
            // the backtrace to the header of the current loop.
            Inst::Split(s) => {
                find_loop_body_end(instructions, s.goto1, header)
                    .or_else(|| {
                        find_loop_body_end(
                            instructions,
                            s.goto2,
                            header,
                        )
                    })?
            }
            // All other instructions are utility instructions
            Inst::Save(x) => x.goto,
            // Instrucitons that do not lead anywhere
            _ => return None,
        };

        // If the next prospective instruction index is the header,
        // the end has been found and should be returned (by exiting
        // this loop)
        if next_index == header {
            break;
        }
        // Otherwise the instruction index needs to be updated so the
        // automata can be followed.
        else {
            passed_inst.insert(inst_index);
            inst_index = next_index;
        }
    }

    // The end index of the loop has been found and will be returned
    Some(inst_index)
}
