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
    pub validate_commands: Vec<(OsString, CommandKind)>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CommandKind {
    Status,
    Diff,
}

fn try_parse_args(args: impl IntoIterator<Item = OsString>) -> Result<Action, lexopt::Error> {
    use lexopt::prelude::*;
    let mut parser = lexopt::Parser::from_iter(args);

    match parser.next()? {
        Some(Value(cmd)) if cmd == "list" => Ok(Action::ListFiles(parse_list_files(&mut parser)?)),
        Some(Value(cmd)) if cmd == "check" => Ok(Action::Check(parse_check(&mut parser)?)),
        Some(Value(cmd)) => Err(format!("Unexpected command {}", cmd.to_string_lossy()))?,
        Some(Short(arg)) => Err(format!("Unexpected argument -{arg} (expecting a command)"))?,
        Some(Long(arg)) => Err(format!("Unexpected argument --{arg} (expecting a command)"))?,
        None => Err("Command 'list' or 'check' must be provided".to_string())?,
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

    while let Some(arg) = parser.next()? {
        match arg {
            Short('j') | Long("jobs") => max_processes = parser.value()?.parse()?,
            Short('I') => placeholder = parser.value()?,
            Short('s') | Long("status") => {
                validate_commands.push((parser.value()?, CommandKind::Status));
            }
            Short('d') | Long("diff") => {
                validate_commands.push((parser.value()?, CommandKind::Diff));
            }
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(Check {
        max_processes,
        placeholder,
        validate_commands,
    })
}

pub fn parse_args(args: impl IntoIterator<Item = OsString>) -> Action {
    match try_parse_args(args) {
        Ok(args) => args,
        Err(err) => {
            dbg!(err);
            // TODO: help text
            std::process::exit(1)
        }
    }
}
