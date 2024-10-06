use std::{
    ffi::{OsStr, OsString},
    io,
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

use crate::{arguments::Mode, errors::Error, World};

// pub struct Processor<'a> {
//     semaphore: Semaphore,
//     placeholder: &'a OsStr,
//     cwd: &'a Path,
// }

// impl<'a> Processor<'a> {
//     pub fn new(semaphore: Semaphore, placeholder: &'a OsStr, cwd: &'a Path) -> Self {
//         Self {
//             semaphore,
//             placeholder,
//             cwd,
//         }
//     }

//     pub async fn process(
//         path: PathBuf,
//         contents: Vec<u8>,
//         format_commands: &[OsString],
//         validate_commands: &[Mode],
//     ) -> (PathBuf, Result<(), Vec<CheckError>>) {

//     }
// }

pub async fn process_file(
    semaphore: &Semaphore,
    placeholder: &OsStr,
    cwd: &Path,
    path: PathBuf,
    contents: Vec<u8>,
    format_commands: &[OsString],
    validate_commands: &[Mode],
) -> (PathBuf, Result<(), Vec<CheckError>>) {
    let result = _process_file(
        semaphore,
        placeholder,
        cwd,
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
    cwd: &Path,
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
        checks.push(process_validation(
            placeholder,
            cwd,
            path,
            &contents,
            command,
        ));
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
    cwd: &Path,
    path: &Path,
    contents: &[u8],
    command: &Mode,
) -> Result<(), CheckError> {
    let mut child = Command::new("sh");
    let cmd_bytes = command.command(placeholder, path);
    child
        .current_dir(cwd)
        .arg("-c")
        .arg(&cmd_bytes)
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
            status_success(cmd_bytes, result.map_err(CheckError::SpawnError)?)
        }
        Mode::Diff(_) => {
            let (write, result) = join!(write_stdin(stdin, contents), child.output());
            write?;
            diff_success(cmd_bytes, result.map_err(CheckError::SpawnError)?, contents)
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

fn status_success(command: OsString, output: Output) -> Result<(), CheckError> {
    if output.status.success() {
        Ok(())
    } else {
        Err(CheckError::StatusFailure {
            command,
            status: output.status,
            output: output.stderr,
        })
    }
}

fn diff_success(command: OsString, output: Output, expected: &[u8]) -> Result<(), CheckError> {
    if !output.status.success() {
        Err(CheckError::StatusFailure {
            status: output.status,
            command,
            output: output.stderr,
        })
    } else if output.stdout != expected {
        Err(CheckError::DiffCheckFailure {
            command,
            output: output.stderr,
        })
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

    #[error("command failed ({status})")]
    StatusFailure {
        status: ExitStatus,
        command: OsString,
        output: Vec<u8>,
    },

    #[error("command produced mismatching output")]
    DiffCheckFailure { command: OsString, output: Vec<u8> },
}

impl CheckError {
    pub fn write_error_message(&self, world: &impl World) -> Result<(), Error> {
        match &self {
            Self::PipeIoError(source) => {
                world.check_failed_info(format_args!(
                    "writing to/from a child process failed ({source})"
                ))?;
            }
            Self::SpawnError(source) => {
                world.check_failed_info(format_args!(
                    "spawning a child process failed ({source})"
                ))?;
            }
            Self::StatusFailure {
                command,
                status,
                output,
            } => {
                world.check_failed_info(format_args!(
                    "command failed `{command}` ({status})",
                    command = command.to_string_lossy()
                ))?;
                if !output.is_empty() {
                    world.stderr_raw_bytes(output)?;
                }
            }
            Self::DiffCheckFailure { command, output } => {
                world.check_failed_info(format_args!(
                    "command output did not match expected source `{command}`",
                    command = command.to_string_lossy()
                ))?;
                if !output.is_empty() {
                    world.stderr_raw_bytes(output)?;
                }
            }
        }

        Ok(())
    }
}
