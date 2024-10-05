use std::io::Write;

use futures::{stream::FuturesUnordered, StreamExt};
use git2::{Repository, RepositoryOpenFlags};
use smol::lock::Semaphore;

use crate::{
    arguments::{Action, Check},
    check::process_file,
    errors::Error,
    repo::{fetch_changed_paths, read_oid},
    world::World,
};

pub fn run(action: Action, world: &mut World<impl Write, impl Write>) -> i32 {
    match try_run(action, world) {
        Ok(_) => 0,
        Err(Error::Git(error)) => {
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
        Err(Error::Write(error)) => {
            writeln!(world.stderr, "Unable to perform IO: {:?}", error).unwrap();
            51
        }
        Err(Error::ChecksFailed()) => {
            writeln!(world.stderr, "One or more checks failed").unwrap();
            1
        }
    }
}

fn try_run(action: Action, world: &mut World<impl Write, impl Write>) -> Result<(), Error> {
    let repo = Repository::open_ext(
        &world.cwd,
        RepositoryOpenFlags::empty(),
        &[] as &[&std::ffi::OsStr],
    )?;

    if let Some(path) = repo.workdir() {
        world.cwd = path.into();
    }

    match action {
        Action::ListFiles(()) => {
            for file in fetch_changed_paths(&repo, world)? {
                world
                    .stdout
                    .write_all(file.0.as_os_str().as_encoded_bytes())?;
                writeln!(world.stdout)?;
            }
            Ok(())
        }
        Action::Check(check) => run_check(check, &repo, world),
    }
}

fn run_check(
    check: Check,
    repo: &Repository,
    world: &mut World<impl Write, impl Write>,
) -> Result<(), Error> {
    let cwd = world.cwd.clone();
    let files = fetch_changed_paths(repo, world)?
        .into_iter()
        .map(|(path, oid)| (path, read_oid(repo, oid, world)));

    let semaphore = Semaphore::new(check.max_processes);

    let failures = {
        let mut futures = FuturesUnordered::new();

        for (path, contents) in files {
            futures.push(process_file(
                &semaphore,
                &check.placeholder,
                &cwd,
                path,
                contents?,
                &check.format_commands,
                &check.validate_commands,
            ));
        }

        let mut failures = 0;
        smol::block_on(async {
            while let Some((path, result)) = futures.next().await {
                match result {
                    Ok(()) => {}
                    Err(errors) => {
                        failures += 1;
                        writeln!(world.stderr, "check(s) failed for path {path:?}")?;
                        for error in errors {
                            error.write_error_message(&mut world.stderr)?;
                        }
                        writeln!(world.stderr)?;
                    }
                }
            }
            Ok::<_, Error>(())
        })?;

        Ok::<_, Error>(failures)
    }?;

    if failures == 0 {
        Ok(())
    } else {
        Err(Error::ChecksFailed())
    }
}
