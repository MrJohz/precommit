use std::path::{Path, PathBuf};

use git2::{Delta, Oid, Repository, RepositoryOpenFlags};

use crate::{errors::Error, World};

pub struct Repo<T> {
    repository: Repository,
    world: T,
}

impl<T> Repo<T>
where
    T: World,
{
    pub fn new(path: &Path, world: T) -> Result<Self, Error> {
        let repository = Repository::open_ext(
            path,
            RepositoryOpenFlags::empty(),
            &[] as &[&std::ffi::OsStr],
        )?;

        Ok(Repo { repository, world })
    }

    pub fn root_dir(&self) -> Result<&Path, Error> {
        let path = self.repository.workdir().ok_or_else(|| {
            git2::Error::new(
                git2::ErrorCode::BareRepo,
                git2::ErrorClass::None,
                "cannot get working directory of repository",
            )
        })?;

        Ok(path)
    }

    pub fn fetch_changed_paths(&self) -> Result<Vec<(PathBuf, Oid)>, Error> {
        let head = self
            .repository
            .head()
            .and_then(|head| head.peel_to_tree())
            .ok();

        let diff = self
            .repository
            .diff_tree_to_index(head.as_ref(), None, None)?;

        let files = diff
            .deltas()
            .filter(|diff| matches!(diff.status(), Delta::Added | Delta::Modified))
            .filter_map(|delta| {
                let oid = delta.new_file().id();
                match delta.new_file().path() {
                    Some(path) => Some((path.to_owned(), oid)),
                    None => {
                        let _ = self.world.warning(format_args!(
                            "Could not find a path for object {oid}, ignoring"
                        ));
                        None
                    }
                }
            })
            .collect();

        Ok(files)
    }
    pub fn read_oid(&self, oid: Oid) -> Result<Vec<u8>, Error> {
        let blob = self.repository.find_blob(oid)?;
        Ok(blob.content().into())
    }
}
