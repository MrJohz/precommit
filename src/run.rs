use std::path::Path;

use futures::{stream::FuturesUnordered, StreamExt};
use smol::lock::Semaphore;

use crate::{
    arguments::{Action, Check},
    check::Processor,
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
        let mut failures = 0;
        let processor = Processor::new(semaphore, &check.placeholder, repo.root_dir()?, world);
        let mut futures = FuturesUnordered::new();

        for (path, contents) in files {
            let contents = match contents {
                Ok(contents) => contents,
                Err(_) => {
                    world.check_failed(format_args!("Could not read file for {path:?}"))?;
                    failures += 1;
                    continue;
                }
            };

            futures.push(processor.process(path, contents, &check.validate_commands));
        }

        failures += smol::block_on(async move {
            let mut failures = 0;
            while let Some(result) = futures.next().await {
                match result {
                    Ok(true) => {}
                    Ok(false) => failures += 1,
                    Err(_) => failures += 1,
                }
            }
            failures
        });

        Ok::<_, Error>(failures)
    }?;

    if failures == 0 {
        Ok(())
    } else {
        Err(Error::ChecksFailed())
    }
}
