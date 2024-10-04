use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Writing to stdout/stderr failed")]
    WriteError(#[from] io::Error),
    #[error("Unexpected failure interacting with git2")]
    GitError(#[from] git2::Error),
}
