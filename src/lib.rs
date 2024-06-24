use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Display,
};

use anyhow::bail;
use dialoguer::{theme::ColorfulTheme, BasicHistory, Confirm, FuzzySelect, History, Input};
use essential_constraint_asm::Op;
use essential_constraint_vm::{
    mut_keys_set, transient_data, Access, BytecodeMapped, OpAccess, ProgramControlFlow, Repeat,
    SolutionAccess, Stack, StateSlotSlice, StateSlots, TransientData,
};
use essential_types::{
    intent::Intent,
    solution::{Solution, SolutionDataIndex},
    ContentAddress, Key, Value, Word,
};

pub use state_builder::StateBuilder;

mod parse_types;
mod state;
mod state_builder;

const PROMPT: &str = "<essential-dbg>";
const PRIMITIVES: &[&str] = &["int", "bool", "b256"];
const COMPOUND: &[&str] = &["array", "tuple"];

pub struct ConstraintDebugger {
    stack: Stack,
    memory: essential_constraint_vm::Memory,
    repeat: Repeat,
    pc: usize,
    code: BytecodeMapped<Op>,
    solution: Solution,
    pre_state: Vec<Vec<Word>>,
    post_state: Vec<Vec<Word>>,
    index: SolutionDataIndex,
}

pub struct Session<'a> {
    solution: &'a Solution,
    index: SolutionDataIndex,
    mutable_keys: HashSet<&'a [Word]>,
    transient_data: TransientData,
    pre: &'a StateSlotSlice,
    post: &'a StateSlotSlice,
    code: &'a mut BytecodeMapped<Op>,
    stack: &'a mut Stack,
    memory: &'a mut essential_constraint_vm::Memory,
    repeat: &'a mut Repeat,
    pc: &'a mut usize,
    last_op: Option<essential_constraint_asm::Constraint>,
    pos: usize,
}

pub enum Outcome {
    ProgramEnd,
    Step,
}

pub async fn run(
    solution: Solution,
    index: SolutionDataIndex,
    intent: Intent,
    constraint: usize,
    state: HashMap<ContentAddress, BTreeMap<Key, Value>>,
) -> anyhow::Result<()> {
    let mut debugger = ConstraintDebugger::new(solution, index, intent, constraint, state).await?;
    let mut session = debugger.start_session();

    let mut out = String::new();

    let mut history = BasicHistory::new().max_entries(20).no_duplicates(true);

    loop {
        let command: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(&format!("{}\n{}", out, PROMPT))
            .history_with(&mut history)
            .interact_text()?;

        match command.as_str() {
            "n" | "next" => session.next(&mut out)?,
            "b" | "back" => session.back(&mut out)?,
            "q" | "quit" | "exit" => break,
            "h" | "help" => {
                out = help_msg();
            }
            "h t" | "help type" | "h type" | "help t" => {
                out = types_msg();
            }
            _ => {
                let mut c = command.split(' ');

                let Some(next_command) = c.next() else {
                    out = format!("Unknown command: {}", command);
                    continue;
                };
                match next_command {
                    "p" | "play" => {
                        let i = c
                            .next()
                            .and_then(|i| i.parse::<usize>().ok())
                            .unwrap_or_default();
                        session.play(i, &mut out)?;
                    }
                    "t" | "type" => {
                        let rest = c.filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ");
                        if rest.is_empty() {
                            let prompt = format!("{}::type", PROMPT);
                            let pos: String = Input::with_theme(&ColorfulTheme::default())
                                .with_prompt(&format!("Enter position\n{}", prompt))
                                .default("0".to_string())
                                .history_with(&mut history)
                                .interact_text()?;
                            let pos: usize = pos.trim().parse().unwrap_or_default();

                            let prompt = format!("{}::{}", prompt, pos);
                            let mut options = PRIMITIVES.to_vec();
                            options.extend_from_slice(COMPOUND);

                            let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                                .with_prompt(&format!("Select type\n{}", prompt))
                                .default(0)
                                .items(&options[..])
                                .interact()?;
                            if PRIMITIVES.contains(&options[selection]) {
                                let input = format!("{} {}", pos, &options[selection]);
                                out = session.parse_type(&input);
                            } else {
                                let prompt = format!("{}::{}", prompt, options[selection]);
                                let input =
                                    match options[selection] {
                                        "array" => {
                                            let selection =
                                                FuzzySelect::with_theme(&ColorfulTheme::default())
                                                    .with_prompt(&format!(
                                                        "Select array type\n{}",
                                                        prompt
                                                    ))
                                                    .default(0)
                                                    .items(PRIMITIVES)
                                                    .interact()?;

                                            let prompt =
                                                format!("{}::{}", prompt, PRIMITIVES[selection]);
                                            let len: String =
                                                Input::with_theme(&ColorfulTheme::default())
                                                    .with_prompt(&format!(
                                                        "Enter array length\n{}",
                                                        prompt
                                                    ))
                                                    .default("1".to_string())
                                                    .history_with(&mut history)
                                                    .interact_text()?;
                                            let len: usize = len.trim().parse().unwrap_or_default();
                                            format!("{} {}[{}]", pos, PRIMITIVES[selection], len)
                                        }
                                        "tuple" => {
                                            let mut fields = String::new();
                                            let mut add_field = true;
                                            while add_field {
                                                let p = format!("{}::{{ {} }}", prompt, fields);
                                                let selection = FuzzySelect::with_theme(
                                                    &ColorfulTheme::default(),
                                                )
                                                .with_prompt(&format!("Select field type\n{}", p))
                                                .default(0)
                                                .items(PRIMITIVES)
                                                .interact()?;

                                                fields.push_str(PRIMITIVES[selection]);

                                                let p = format!("{}::{{ {} }}", prompt, fields);

                                                add_field =
                                                    Confirm::with_theme(&ColorfulTheme::default())
                                                        .with_prompt(&format!(
                                                            "Do you want to add another field?\n{}",
                                                            p
                                                        ))
                                                        .default(true)
                                                        .interact()?;
                                                if add_field {
                                                    fields.push_str(", ");
                                                }
                                            }
                                            format!("{} {{ {} }}", pos, fields)
                                        }
                                        _ => unreachable!(),
                                    };

                                let force_hex = Confirm::with_theme(&ColorfulTheme::default())
                                    .with_prompt(&format!(
                                        "Do you want to force HEX formatting?\n{}",
                                        prompt
                                    ))
                                    .default(false)
                                    .interact()?;
                                let input = if force_hex {
                                    format!("{} HEX", input)
                                } else {
                                    input
                                };
                                history.write(&format!("t {}", input));
                                out = session.parse_type(&input);
                            }
                        } else {
                            out = session.parse_type(&rest);
                        }
                    }
                    _ => {
                        out = format!("Unknown command: {}", command);
                    }
                }
            }
        }
    }

    Ok(())
}

fn end(out: &mut String) {
    out.push_str("\nProgram ended");
}

fn help_msg() -> String {
    r#"Commands:
    n | next: Step forward
    b | back: Step back
    p | play [i]: Play to ith op
    t | type <i> [type]: Parse the ith word in the stack as the given type. See `help type` for more info.
    q | quit | exit: Quit
    h | help: Show this message
    "#
    .to_string()
}

fn types_msg() -> String {
    r#"Primitives: int, bool, b256
    Arrays: primitive[] (e.g. int[])
    Tuple: { primitive, primitive, ... } (e.g. {int, bool, b256})
    Note that nesting types is not currently supported.
    To parse a section of the stack as a type, use `t <i> [type]` 
    e.g. `t 1 int[2]` to parse the second and third word as ints.
    `b256` is always printed as hex. 
    You can force hex formatting by adding `HEX` to the end of the command.
    e.g. `t 1 int HEX`
    "#
    .to_string()
}

impl ConstraintDebugger {
    pub async fn new(
        solution: Solution,
        index: SolutionDataIndex,
        intent: Intent,
        constraint: usize,
        state: HashMap<ContentAddress, BTreeMap<Key, Value>>,
    ) -> anyhow::Result<Self> {
        let slots = state::read_state(&solution, index, &intent, state.clone()).await?;

        let Some(code) = intent.constraints.get(constraint).cloned() else {
            bail!("No constraint found");
        };

        let code = BytecodeMapped::try_from_bytes(code)?;
        let s = Self {
            stack: Default::default(),
            memory: Default::default(),
            repeat: Default::default(),
            pc: 0,
            code,
            solution,
            pre_state: slots.pre,
            post_state: slots.post,
            index,
        };
        Ok(s)
    }

    pub fn start_session(&mut self) -> Session<'_> {
        let mutable_keys = mut_keys_set(&self.solution, self.index);
        let transient_data = transient_data(&self.solution);
        Session {
            code: &mut self.code,
            stack: &mut self.stack,
            memory: &mut self.memory,
            repeat: &mut self.repeat,
            pc: &mut self.pc,
            last_op: None,
            solution: &self.solution,
            index: self.index,
            mutable_keys,
            transient_data,
            pre: &self.pre_state,
            post: &self.post_state,
            pos: 0,
        }
    }
}

fn handle_outcome(outcome: Outcome, out: &mut String) {
    match outcome {
        Outcome::ProgramEnd => end(out),
        Outcome::Step => (),
    }
}

impl Session<'_> {
    pub fn reset_session(&mut self) {
        *self.stack = Default::default();
        *self.memory = Default::default();
        *self.repeat = Default::default();
        *self.pc = 0;
        self.pos = 0;
    }

    pub fn next(&mut self, out: &mut String) -> anyhow::Result<()> {
        let outcome = self.step_forward()?;

        *out = format!("{}", self);

        handle_outcome(outcome, out);
        Ok(())
    }

    pub fn back(&mut self, out: &mut String) -> anyhow::Result<()> {
        let pos = self.pos.saturating_sub(1);
        self.reset_session();
        let outcome = self.play_to(pos)?;
        *out = format!("{}", self);

        handle_outcome(outcome, out);
        Ok(())
    }

    pub fn play(&mut self, i: usize, out: &mut String) -> anyhow::Result<()> {
        self.reset_session();
        let outcome = self.play_to(i)?;
        *out = format!("{}", self);

        handle_outcome(outcome, out);
        Ok(())
    }

    pub fn step_forward(&mut self) -> anyhow::Result<Outcome> {
        let Self {
            code,
            stack,
            memory,
            repeat,
            pc,
            last_op,
            solution,
            index,
            mutable_keys,
            transient_data,
            pre,
            post,
            pos,
        } = self;

        let access = Access {
            solution: SolutionAccess::new(solution, *index, mutable_keys, transient_data),
            state_slots: StateSlots { pre, post },
        };

        let op = (&**code).op_access(**pc);

        let op = match op {
            Some(Ok(op)) => op,
            Some(Err(err)) => {
                // Handle error
                bail!("Error: {:?}", err);
            }
            None => {
                // end of program
                return Ok(Outcome::ProgramEnd);
            }
        };

        last_op.replace(op);

        let result = essential_constraint_vm::step_op(access, op, stack, memory, **pc, repeat)?;
        *pos += 1;

        match result {
            Some(ProgramControlFlow::Pc(new_pc)) => {
                **pc = new_pc;
                Ok(Outcome::Step)
            }
            Some(ProgramControlFlow::Halt) => Ok(Outcome::ProgramEnd),
            None => {
                **pc += 1;
                Ok(Outcome::Step)
            }
        }
    }

    pub fn play_to(&mut self, i: usize) -> anyhow::Result<Outcome> {
        let mut out = None;
        let i = if i == 0 { 1 } else { i };
        for _ in 0..i {
            match self.step_forward()? {
                Outcome::ProgramEnd => return Ok(Outcome::ProgramEnd),
                Outcome::Step => {
                    out = Some(Outcome::Step);
                }
            }
        }
        let Some(out) = out else {
            bail!("Program didn't run");
        };
        Ok(out)
    }

    pub fn parse_type(&self, ty: &str) -> String {
        parse_types::parse_type(&self.stack[..], ty)
    }
}

impl Display for Session<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(op) = &self.last_op {
            writeln!(f, "Op: {:?}", op)?;
        }
        writeln!(f, "  ├── {:?}\n  └── {:?}", self.stack, self.memory)
    }
}
