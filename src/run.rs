use std::path::Path;

use futures::{stream::FuturesUnordered, StreamExt};
use smol::lock::Semaphore;

use crate::{
    arguments::{Action, Check},
    check::process_file,
    errors::Error,
    repo::Repo,
    world::World,
};

pub fn run(cwd: &Path, action: Action, world: &impl World) -> i32 {
    match try_run(cwd, action, world) {
        Ok(_) => 0,
        Err(Error::Git(error)) => {
            world
                .error(format_args!(
                    "A git operation failed unexpectedly (class {2:?}, code {1:?}): {0}",
                    error.message(),
                    error.code(),
                    error.class()
                ))
                .unwrap();
            50
        }
        Err(Error::Write(error)) => {
            world
                .error(format_args!("Unable to perform IO: {:?}", error))
                .unwrap();
            51
        }
        Err(Error::ChecksFailed()) => {
            world
                .error(format_args!("One or more checks failed"))
                .unwrap();
            1
        }
    }
}

fn try_run(cwd: &Path, action: Action, world: &impl World) -> Result<(), Error> {
    let repo = Repo::new(cwd, world.clone())?;

    match action {
        Action::ListFiles(()) => {
            for file in repo.fetch_changed_paths()? {
                world.output(file.0.as_os_str().as_encoded_bytes())?;
                world.output(b"\n")?;
            }
            Ok(())
        }
        Action::Check(check) => run_check(check, &repo, world),
    }
}

fn run_check(check: Check, repo: &Repo<impl World>, world: &impl World) -> Result<(), Error> {
    let files = repo
        .fetch_changed_paths()?
        .into_iter()
        .map(|(path, oid)| (path, repo.read_oid(oid)));

    let semaphore = Semaphore::new(check.max_processes);

    let failures = {
        let mut futures = FuturesUnordered::new();

        for (path, contents) in files {
            futures.push(process_file(
                &semaphore,
                &check.placeholder,
                repo.root_dir()?,
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
                        world.error(format_args!("check(s) failed for path {path:?}"))?;
                        for error in errors {
                            error.write_error_message(world)?;
                        }
                        world.stderr_raw_bytes(b"\n")?;
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
