use std::{
    ffi::{OsStr, OsString},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{ExitStatus, Output, Stdio},
};

use futures::{join, stream::FuturesUnordered, StreamExt};
use smol::{
    io::AsyncWriteExt,
    lock::Semaphore,
    process::{ChildStdin, Command},
};
use thiserror::Error;

use crate::{arguments::Mode, errors::Error};

pub async fn process_file(
    semaphore: &Semaphore,
    placeholder: &OsStr,
    path: PathBuf,
    contents: Vec<u8>,
    format_commands: &[OsString],
    validate_commands: &[Mode],
) -> (PathBuf, Result<(), Vec<CheckError>>) {
    let result = _process_file(
        semaphore,
        placeholder,
        &path,
        contents,
        format_commands,
        validate_commands,
    )
    .await;

    (path, result)
}

async fn _process_file(
    semaphore: &Semaphore,
    placeholder: &OsStr,
    path: &Path,
    contents: Vec<u8>,
    format_commands: &[OsString],
    validate_commands: &[Mode],
) -> Result<(), Vec<CheckError>> {
    let _lock = semaphore.acquire().await;

    let contents = process_formatting(path, contents, format_commands)
        .await
        .map_err(|err| vec![err])?;

    let checks = FuturesUnordered::new();

    for command in validate_commands {
        checks.push(process_validation(placeholder, path, &contents, command));
    }

    let errors: Vec<_> = checks
        .filter_map(|check| async {
            match check {
                Ok(()) => None,
                Err(err) => Some(err),
            }
        })
        .collect()
        .await;

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

async fn process_formatting(
    _path: &Path,
    contents: Vec<u8>,
    format_commands: &[OsString],
) -> Result<Vec<u8>, CheckError> {
    if format_commands.is_empty() {
        return Ok(contents);
    }

    unimplemented!("implement formatting");
}

async fn process_validation(
    placeholder: &OsStr,
    path: &Path,
    contents: &[u8],
    command: &Mode,
) -> Result<(), CheckError> {
    let mut child = Command::new("sh");
    child
        .arg("-c")
        .arg(command.command(placeholder, path))
        .stdin(Stdio::piped())
        .stderr(Stdio::piped());

    if matches!(command, Mode::Diff(_)) {
        child.stdout(Stdio::piped());
    } else {
        child.stdout(Stdio::null());
    }

    let mut child = child.spawn().map_err(CheckError::SpawnError)?;
    let stdin = child.stdin.take().expect("stdin is not a pipe");

    match command {
        Mode::Status(_) => {
            let (write, result) = join!(write_stdin(stdin, contents), child.output());
            write?;
            status_success(result.map_err(CheckError::SpawnError)?)
        }
        Mode::Diff(_) => {
            let (write, result) = join!(write_stdin(stdin, contents), child.output());
            write?;
            diff_success(result.map_err(CheckError::SpawnError)?, contents)
        }
    }
}

async fn write_stdin(mut stdin: ChildStdin, contents: &[u8]) -> Result<(), CheckError> {
    // stdin will automatically get dropped here, which closes the stdin pipe and flushes
    // the data.
    stdin
        .write_all(contents)
        .await
        .map_err(CheckError::PipeIoError)
}

fn status_success(output: Output) -> Result<(), CheckError> {
    if output.status.success() {
        Ok(())
    } else {
        Err(CheckError::StatusFailure(output.status, output.stderr))
    }
}

fn diff_success(output: Output, expected: &[u8]) -> Result<(), CheckError> {
    if !output.status.success() {
        Err(CheckError::StatusFailure(output.status, output.stderr))
    } else if output.stdout != expected {
        Err(CheckError::DiffCheckFailure(output.stderr))
    } else {
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum CheckError {
    #[error("writing to/from a child process failed")]
    PipeIoError(#[source] io::Error),

    #[error("spawning a child process failed")]
    SpawnError(#[source] io::Error),

    #[error("command failed ({0})")]
    StatusFailure(ExitStatus, Vec<u8>),

    #[error("command produced mismatching output")]
    DiffCheckFailure(Vec<u8>),
}

impl CheckError {
    pub fn write_error_message(&self, stderr: &mut impl Write) -> Result<(), Error> {
        match &self {
            Self::PipeIoError(source) => {
                writeln!(stderr, "writing to/from a child process failed ({source})")?
            }
            Self::SpawnError(source) => {
                writeln!(stderr, "spawning a child process failed ({source})")?
            }
            Self::StatusFailure(status, output) => {
                writeln!(stderr, "command failed ({status})")?;
                if !output.is_empty() {
                    stderr.write_all(output)?;
                }
            }
            Self::DiffCheckFailure(output) => {
                writeln!(stderr, "command produced mismatching output")?;
                if !output.is_empty() {
                    stderr.write_all(output)?;
                }
            }
        }

        Ok(())
    }
}
