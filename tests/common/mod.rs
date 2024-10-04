use std::{
    env,
    io::{self, Write},
    path::PathBuf,
    process::Output,
};

use tempfile::TempDir;

static EXE_NAME: &str = "precommit";

pub struct Expectations {
    output: Output,
}

impl Expectations {
    fn debug_output(&self) {
        let mut stdout = io::stdout().lock();
        let _ = writeln!(stdout, ".- STATUS: {}", self.output.status);
        let _ = writeln!(stdout, "|- STDOUT ------");
        if self.output.stdout == b"" {
            let _ = writeln!(stdout, "| (no output) ");
        } else {
            for line in self.output.stdout[..self.output.stdout.len() - 1].split(|c| *c == b'\n') {
                let _ = stdout.write_all(b"| ");
                let _ = stdout.write_all(line);
                let _ = stdout.write_all(b"\n");
            }
        }
        let _ = writeln!(stdout, "|- STDERR ------");
        if self.output.stderr == b"" {
            let _ = writeln!(stdout, "| (no output) ");
        } else {
            for line in self.output.stderr[..self.output.stderr.len() - 1].split(|c| *c == b'\n') {
                let _ = stdout.write_all(b"| ");
                let _ = stdout.write_all(line);
                let _ = stdout.write_all(b"\n");
            }
        }
        let _ = writeln!(stdout, "'---------------");
    }

    pub fn is_failure(&self, status_code: i32) -> &Self {
        if !self.output.status.success() && self.output.status.code() == Some(status_code) {
            return self;
        }

        self.debug_output();
        panic!(
            "assertion failed ({} !== {})",
            self.output.status, status_code
        )
    }

    pub fn is_success(&self) -> &Self {
        if self.output.status.success() {
            return self;
        }

        self.debug_output();
        panic!(
            "assertion failed (success exit, got failure {})",
            self.output.status
        )
    }

    pub fn stdout_equals(&self, stdout: impl AsRef<[u8]>) -> &Self {
        if self.output.stdout == stdout.as_ref() {
            return self;
        }

        self.debug_output();
        panic!("assertion failed (stdout did not match)");
    }

    pub fn stderr_equals(&self, stderr: impl AsRef<[u8]>) -> &Self {
        if self.output.stderr == stderr.as_ref() {
            return self;
        }

        self.debug_output();
        panic!("assertion failed (stderr did not match)");
    }
}

pub fn expect(output: Output) -> Expectations {
    Expectations { output }
}

pub fn exe() -> PathBuf {
    env::var_os("CARGO_BIN_PATH")
        .map(PathBuf::from)
        .or_else(|| {
            env::current_exe().ok().map(|mut path| {
                path.pop();
                if path.ends_with("deps") {
                    path.pop();
                }
                path
            })
        })
        .unwrap_or_else(|| panic!("CARGO_BIN_PATH wasn't set. Cannot continue running test"))
        .join(EXE_NAME)
}

pub fn tmpdir() -> TempDir {
    let dir = TempDir::new().expect("Could not create temp dir");

    env::set_current_dir(&dir).expect("Could not change directory to temp dir");

    dir
}

#[macro_export]
macro_rules! exec {
    (self, $($arg:expr),*) => {
        exec!($crate::common::exe(), $($arg),*)
    };
    ($cmd:expr, $($arg:expr),*) => {
        {
            let output = ::std::process::Command::new($cmd)
                $(.arg($arg))*
                .output().expect("failed to execute process");
            expect(output)
        }
    }
}
