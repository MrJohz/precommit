use std::{ffi::OsString, num::NonZero, thread};

#[derive(Debug)]
pub enum Action {
    ListFiles(()),
    Check(Check),
}

#[derive(Debug)]
pub struct Check {
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

fn parse_args() -> Result<Action, lexopt::Error> {
    use lexopt::prelude::*;
    let mut parser = lexopt::Parser::from_env();

    match parser.next()? {
        Some(Value(cmd)) if cmd == "list" => Ok(Action::ListFiles(parse_list_files(&mut parser)?)),
        Some(Value(cmd)) if cmd == "check" => Ok(Action::Check(parse_check(&mut parser)?)),
        Some(Value(cmd)) => Err(format!("Unexpected command {}", cmd.to_string_lossy()))?,
        Some(Short(arg)) => Err(format!("Unexpected argument -{arg} (expecting a command)"))?,
        Some(Long(arg)) => Err(format!("Unexpected argument --{arg} (expecting a command)"))?,
        None => Err(format!("Command 'list' or 'check' must be provided"))?,
    }
}

fn parse_list_files(parser: &mut lexopt::Parser) -> Result<(), lexopt::Error> {
    if let Some(arg) = parser.next()? {
        return Err(arg.unexpected());
    }

    Ok(())
}

fn parse_check(parser: &mut lexopt::Parser) -> Result<Check, lexopt::Error> {
    use lexopt::prelude::*;

    let mut max_processes = thread::available_parallelism()
        .map(NonZero::get)
        .unwrap_or(1);
    let mut placeholder = OsString::from("{}");
    let mut validate_commands = Vec::new();
    let mut format_commands = Vec::new();

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

    Ok(Check {
        max_processes,
        placeholder,
        validate_commands,
        format_commands,
    })
}

pub fn parse() -> Action {
    match parse_args() {
        Ok(args) => args,
        Err(err) => {
            dbg!(err);
            // TODO: help text
            std::process::exit(1)
        }
    }
}
