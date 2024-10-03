use std::{ffi::OsString, num::NonZero, thread};

#[derive(Debug)]
pub struct Args {
    pub max_processes: usize,
    pub placeholder: OsString,
    pub validate_commands: Vec<Mode>,
    pub format_commands: Vec<OsString>,
}

#[derive(Debug)]
pub enum Mode {
    Status(OsString),
    Diff(OsString),
}

fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut max_processes = thread::available_parallelism()
        .map(NonZero::get)
        .unwrap_or(1);
    let mut placeholder = OsString::from("{}");
    let mut validate_commands = Vec::new();
    let mut format_commands = Vec::new();

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('j') | Long("jobs") => max_processes = parser.value()?.parse()?,
            Short('I') => placeholder = parser.value()?,
            Short('s') | Long("status") => {
                validate_commands.push(Mode::Status(parser.value()?));
            }
            Short('d') | Long("diff") => {
                validate_commands.push(Mode::Diff(parser.value()?));
            }
            Short('f') | Long("format") => {
                format_commands.push(parser.value()?);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(Args {
        max_processes,
        placeholder,
        validate_commands,
        format_commands,
    })
}

pub fn parse() -> Args {
    match parse_args() {
        Ok(args) => args,
        Err(err) => {
            dbg!(err);
            // TODO: help text
            std::process::exit(1)
        }
    }
}
