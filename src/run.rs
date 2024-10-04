use std::io::Write;

use crate::{arguments::Action, errors::Error, repo::fetch_changed_paths, world::World};

pub fn run(action: Action, world: &mut World<impl Write, impl Write>) -> i32 {
    match try_run(action, world) {
        Ok(status) => status,
        Err(Error::GitError(error)) => {
            writeln!(
                world.stderr,
                "An git operation failed unexpectedly (class {2:?}, code {1:?}): {0} ",
                error.message(),
                error.code(),
                error.class()
            )
            .unwrap();
            50
        }
        Err(Error::WriteError(error)) => {
            writeln!(world.stderr, "Unable to perform IO: {:?}", error).unwrap();
            51
        }
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

fn try_run(action: Action, world: &mut World<impl Write, impl Write>) -> Result<i32, Error> {
    let repo = git2::Repository::open(dbg!(&world.cwd))?;

    match action {
        Action::ListFiles(()) => {
            for file in fetch_changed_paths(&repo, world)? {
                world
                    .stdout
                    .write_all(file.0.as_os_str().as_encoded_bytes())?;
                world.stdout.write_all(b"\n")?;
            }
        }
        Action::Check(validate) => {
            dbg!(validate);
        }
    }
    Ok(0)
}
