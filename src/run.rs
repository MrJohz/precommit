use std::{io::Write, path::Path};

use crate::{arguments::Action, errors::Error, repo::fetch_changed_paths};

pub fn run(
    action: Action,
    cwd: &impl AsRef<Path>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    match try_run(action, cwd, stdout, stderr) {
        Ok(status) => status,
        Err(err) => panic!("{1}: {:?}", err, "better error handling is need, pronto!"),
    }

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

fn try_run(
    action: Action,
    cwd: &impl AsRef<Path>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> Result<i32, Error> {
    let cwd = cwd.as_ref();
    let repo = git2::Repository::open(cwd)?;

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
    Ok(0)
}
