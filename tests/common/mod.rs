#![allow(dead_code)]

use std::{
    env,
    fmt::Debug,
    fs::{create_dir_all, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use bstr::ByteSlice;
use git2::{Signature, Time};
use precommit::WriterWorld;
use tempfile::TempDir;

static EXE_NAME: &str = "precommit";

pub struct Expectations {
    code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl Expectations {
    fn debug_output(&self) {
        let mut stdout = io::stdout().lock();
        let _ = writeln!(stdout, ".- STATUS: {}", self.code);
        let _ = writeln!(stdout, "|- STDOUT ------");
        if self.stdout == b"" {
            let _ = writeln!(stdout, "| (no output) ");
        } else {
            for line in self.stdout[..self.stdout.len() - 1].split(|c| *c == b'\n') {
                let _ = stdout.write_all(b"| ");
                let _ = stdout.write_all(line);
                let _ = stdout.write_all(b"\n");
            }
        }
        let _ = writeln!(stdout, "|- STDERR ------");
        if self.stderr == b"" {
            let _ = writeln!(stdout, "| (no output) ");
        } else {
            for line in self.stderr[..self.stderr.len() - 1].split(|c| *c == b'\n') {
                let _ = stdout.write_all(b"| ");
                let _ = stdout.write_all(line);
                let _ = stdout.write_all(b"\n");
            }
        }
        let _ = writeln!(stdout, "'---------------");
    }

    pub fn is_failure(&self, status_code: i32) -> &Self {
        if self.code == status_code {
            return self;
        }

        self.debug_output();
        panic!("assertion failed ({} !== {})", self.code, status_code)
    }

    pub fn is_success(&self) -> &Self {
        if self.code == 0 {
            return self;
        }

        self.debug_output();
        panic!("assertion failed (success exit, got failure {})", self.code)
    }

    pub fn stdout_equals(&self, stdout: impl AsRef<[u8]>) -> &Self {
        if self.stdout == stdout.as_ref() {
            return self;
        }

        self.debug_output();
        panic!("assertion failed (stdout did not match)");
    }

    pub fn stderr_equals(&self, stderr: impl AsRef<[u8]>) -> &Self {
        if self.stderr == stderr.as_ref() {
            return self;
        }

        self.debug_output();
        panic!("assertion failed (stderr did not match)");
    }

    pub fn stderr_contains(&self, stderr: impl AsRef<[u8]> + Debug) -> &Self {
        if self.stderr.contains_str(stderr.as_ref()) {
            return self;
        }

        self.debug_output();
        panic!("assertion failed (stderr did not contain {stderr:?})")
    }

    pub fn stderr_not_contains(&self, stderr: impl AsRef<[u8]> + Debug) -> &Self {
        if !self.stderr.contains_str(stderr.as_ref()) {
            return self;
        }

        self.debug_output();
        panic!("assertion failed (stderr contained {stderr:?})")
    }
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

pub fn dir() -> (TempDir, Dir) {
    let dir = TempDir::new().expect("Could not create temp dir");
    let path = dir.path().into();

    (dir, Dir { path })
}

pub struct Dir {
    path: PathBuf,
}

impl Dir {
    pub fn exec_self<'a>(&self, args: impl IntoIterator<Item = &'a str>) -> Expectations {
        let args = vec![exe().into_os_string()]
            .into_iter()
            .chain(args.into_iter().map(|each| each.into()));

        let action = precommit::parse_args(args);
        let stdout = Vec::new();
        let stderr = Vec::new();
        let world = WriterWorld::new(stdout, stderr);
        let code = precommit::run(&self.path, action, &world);

        let (stdout, stderr) = world.outputs();

        Expectations {
            code,
            stdout,
            stderr,
        }
    }

    pub fn git_init(&self) {
        git2::Repository::init(&self.path).expect("could not init repository");
    }

    pub fn git_add(&self, path: impl AsRef<Path>) {
        let path = path.as_ref();

        let repo = git2::Repository::open(&self.path).expect("could not open repository");
        let mut index = repo.index().expect("could not fetch index");
        index.add_path(path).expect("could not add file");
        index.write().expect("could not write index")
    }

    pub fn git_commit(&self) {
        let repo = git2::Repository::open(&self.path).expect("could not open repository");

        let now = Time::new(1728076698, 0);
        let signature = Signature::new("dummy test", "test@test.test", &now)
            .expect("invalid signature created");

        let mut index = repo.index().unwrap();
        let oid = index.write_tree().unwrap();
        let tree = repo.find_tree(oid).unwrap();

        let parent_commit = repo.head().and_then(|head| head.peel_to_commit()).ok();
        let result = match parent_commit {
            Some(parent) => repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "dummy message",
                &tree,
                &[&parent],
            ),
            None => repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "dummy message",
                &tree,
                &[],
            ),
        };

        result.expect("could not commit files");
    }

    pub fn subdir(&self, path: impl AsRef<Path>) -> Self {
        let subdir = self.path.join(path.as_ref());
        create_dir_all(&subdir).expect("could not create parent directory for path");

        Self { path: subdir }
    }

    pub fn file(&self, path: impl AsRef<Path>, contents: impl Into<Vec<u8>>) {
        let path = self.path.join(path.as_ref());
        if let Some(path) = path.parent() {
            create_dir_all(path).expect("could not create parent directory for path");
        }

        File::create(path)
            .expect("could not create file")
            .write_all(&contents.into())
            .expect("could not write file contents to file");
    }

    pub fn read(&self, path: impl AsRef<Path>) -> String {
        let path = self.path.join(path.as_ref());
        let mut buf = String::new();
        File::open(path)
            .expect("could not open file")
            .read_to_string(&mut buf)
            .expect("could not read fie contents");
        buf
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
