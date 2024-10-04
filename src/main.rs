mod arguments;

use std::{
    ffi::OsString,
    io::{self, Write},
    path::PathBuf,
    process::Stdio,
};

use anyhow::{anyhow, Result};
use arguments::Action;
use bstr::{ByteSlice, ByteVec};
use futures::{join, stream::FuturesUnordered, StreamExt};
use git2::{Delta, DiffDelta, Oid, Repository};
use smol::{io::AsyncWriteExt, lock::Semaphore, process::Command};

#[derive(Debug)]
struct BlobbedFile {
    pub filename: PathBuf,
    pub contents: Vec<u8>,
}

impl BlobbedFile {
    pub fn new(repo: &Repository, delta: DiffDelta<'_>) -> Result<Self> {
        let blob = repo.find_blob(delta.new_file().id())?;
        Ok(Self {
            filename: delta
                .old_file()
                .path()
                .ok_or_else(|| anyhow!("Invalid path"))?
                .into(),
            contents: blob.content().to_owned(),
        })
    }
}

async fn validate_file_status(
    semaphore: &Semaphore,
    file: &BlobbedFile,
    command: OsString,
) -> Result<bool> {
    let result = {
        let _guard = semaphore.acquire().await;
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let mut stdin = child.stdin.take().expect("stdin is not a pipe");
        let (_, result) = join!(stdin.write_all(&file.contents), child.status());
        result?
    };

    Ok(result.success())
}

// async fn validate_file(semaphore: &Semaphore, file: BlobbedFile, arguments: &Args) -> Result<()> {
//     let mut futures = FuturesUnordered::new();
//     for validation in &arguments.validate_commands {
//         match validation {
//             arguments::Mode::Status(command) => {
//                 let command = command
//                     .as_encoded_bytes()
//                     .replace(
//                         arguments.placeholder.as_encoded_bytes(),
//                         file.filename.as_os_str().as_encoded_bytes(),
//                     )
//                     .into_os_string()?;
//                 futures.push(validate_file_status(semaphore, &file, command))
//             }
//             _ => panic!("not supported!"),
//         }
//     }

//     while let Some(result) = futures.next().await {
//         dbg!(result)?;
//     }
//     Ok(())
// }

fn fetch_changed_paths(repo: &Repository) -> Result<Vec<(PathBuf, Oid)>> {
    let head = repo.head()?.peel_to_tree()?;
    let diff = repo.diff_tree_to_index(Some(&head), None, None)?;

    let files = diff
        .deltas()
        .filter(|diff| matches!(diff.status(), Delta::Added | Delta::Modified))
        .filter_map(|delta| {
            let oid = delta.new_file().id();
            match delta.new_file().path() {
                Some(path) => Some((path.to_owned(), oid)),
                None => {
                    eprintln!("Could not find a path for object {oid}, ignoring");
                    None
                }
            }
        })
        .collect();

    Ok(files)
}

fn main() -> Result<()> {
    let action = arguments::parse();
    let repo = git2::Repository::open(".").unwrap();
    let mut stdout = io::stdout().lock();
    match action {
        Action::ListFiles(()) => {
            for file in fetch_changed_paths(&repo)? {
                stdout.write_all(file.0.as_os_str().as_encoded_bytes())?;
                stdout.write_all(b"\n")?;
            }
        }
        Action::Check(validate) => {
            dbg!(validate);
        }
    }

    Ok(())
    // let arguments = dbg!(arguments::parse());
    // let repo = git2::Repository::open("/tmp/demo-git").unwrap();
    // let head = repo.head().unwrap().peel_to_tree().unwrap();
    // let diff = repo.diff_tree_to_index(Some(&head), None, None).unwrap();
    // let deltas = diff
    //     .deltas()
    //     .filter(|diff| matches!(diff.status(), Delta::Added | Delta::Modified))
    //     .map(|delta| BlobbedFile::new(&repo, delta));

    // let semaphore = Semaphore::new(dbg!(arguments.max_processes));

    // for file in deltas {
    //     smol::block_on(async {
    //         dbg!(validate_file(&semaphore, file.unwrap(), &arguments)
    //             .await
    //             .unwrap());
    //     });
    // }

    // Ok(())
}
