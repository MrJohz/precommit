use std::{
    env,
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

use crate::{arguments::CommandKind, errors::Error, World};

pub struct Processor<'a, W: World> {
    semaphore: Semaphore,
    placeholder: &'a OsStr,
    cwd: &'a Path,
    world: &'a W,
}

impl<'a, W: World> Processor<'a, W> {
    pub fn new(semaphore: Semaphore, placeholder: &'a OsStr, cwd: &'a Path, world: &'a W) -> Self {
        Self {
            semaphore,
            placeholder,
            cwd,
            world,
        }
    }

    pub async fn process(
        &'a self,
        path: PathBuf,
        contents: Vec<u8>,
        commands: &'a [(OsString, CommandKind)],
    ) -> Result<bool, Error> {
        let checks = FuturesUnordered::new();

        for (command, kind) in commands {
            checks.push(self.run_check(command, kind, &path, &contents));
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
            Ok(true)
        } else {
            self.world
                .check_failed(format_args!("check(s) failed for path {path:?}"))?;
            for error in errors {
                error.write_error_message(self.world)?;
            }
            self.world.stderr_raw_bytes(b"\n")?;
            Ok(false)
        }
    }

    async fn run_check(
        &self,
        command: &OsStr,
        kind: &CommandKind,
        path: &Path,
        contents: &[u8],
    ) -> Result<(), CheckError> {
        let _guard = self.semaphore.acquire().await;

        let command = expand_command_string(command, self.placeholder, path);
        let output = self.run_command(&command, kind, contents).await?;

        match kind {
            _ if !output.status.success() => Err(CheckError::StatusFailure {
                command,
                status: output.status,
                output: output.stderr,
            }),
            CommandKind::Diff if output.stdout != contents => Err(CheckError::DiffCheckFailure {
                command,
                output: output.stderr,
            }),
            _ => Ok(()),
        }
    }

    async fn run_command(
        &self,
        command: &OsStr,
        kind: &CommandKind,
        contents: &[u8],
    ) -> Result<Output, CheckError> {
        let mut child = shell()?;
        child
            .current_dir(self.cwd)
            .arg(command)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped());

        match kind {
            CommandKind::Diff => child.stdout(Stdio::piped()),
            CommandKind::Status => child.stdout(Stdio::null()),
        };

        let mut child = child.spawn().map_err(CheckError::SpawnError)?;
        let stdin = child.stdin.take().expect("stdin is not a pipe");

        let (write, output) = join!(write_stdin(stdin, contents), child.output());

        let output = dbg!(output.map_err(CheckError::PipeIoError)?);
        write?;

        Ok(output)
    }
}

fn shell() -> Result<Command, CheckError> {
    if cfg!(windows) {
        let shell_name = env::var_os("ComSpec")
            .or_else(|| {
                env::var_os("SystemRoot")
                    .map(|root| PathBuf::from(root).join("System32").join("cmd.exe").into())
            })
            .ok_or(CheckError::NoShell())?;
        let mut command = Command::new(shell_name);
        command.arg("/c");
        Ok(command)
    } else {
        let mut command = Command::new("/bin/sh");
        command.arg("-c");
        Ok(command)
    }
}

fn expand_command_string(command: &OsStr, placeholder: &OsStr, path: &Path) -> OsString {
    use bstr::ByteSlice;
    // TODO: make this work for Windows as well
    use std::os::unix::ffi::OsStringExt;

    let command = command.as_encoded_bytes().replace(
        placeholder.as_encoded_bytes(),
        path.as_os_str().as_encoded_bytes(),
    );

    OsString::from_vec(command)
}

async fn write_stdin(mut stdin: ChildStdin, contents: &[u8]) -> Result<(), CheckError> {
    // stdin will automatically get dropped here, which closes the stdin pipe and flushes
    // the data.
    stdin
        .write_all(contents)
        .await
        .map_err(CheckError::PipeIoError)
}

#[derive(Error, Debug)]
pub enum CheckError {
    #[error("could not find shell")]
    NoShell(),

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
            Self::NoShell() => world.check_failed_info(format_args!(
                "could not find a valid shell to execute command with"
            ))?,
            Self::PipeIoError(source) => world.check_failed_info(format_args!(
                "writing to/from a child process failed ({source})"
            ))?,
            Self::SpawnError(source) => world
                .check_failed_info(format_args!("spawning a child process failed ({source})"))?,
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
