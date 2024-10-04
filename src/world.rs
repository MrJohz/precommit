use std::{io::Write, path::PathBuf};

pub struct World<Stdout, Stderr> {
    pub cwd: PathBuf,
    pub stdout: Stdout,
    pub stderr: Stderr,
}

impl<Stdout, Stderr> World<Stdout, Stderr> {
    pub fn new(cwd: PathBuf, stdout: Stdout, stderr: Stderr) -> Self
    where
        Stdout: Write,
        Stderr: Write,
    {
        Self {
            cwd,
            stdout,
            stderr,
        }
    }

    pub fn outputs(self) -> (Stdout, Stderr) {
        let Self {
            cwd: _,
            stdout,
            stderr,
        } = self;
        (stdout, stderr)
    }
}
