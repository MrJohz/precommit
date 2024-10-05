use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Writing to stdout/stderr failed")]
    Write(#[from] io::Error),
    #[error("Unexpected failure interacting with git2")]
    Git(#[from] git2::Error),
    #[error("Some checks failed")]
    ChecksFailed(),
}
