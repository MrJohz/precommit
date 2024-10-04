use std::path::PathBuf;

use git2::{Delta, Oid, Repository};

use crate::errors::Error;

pub fn fetch_changed_paths(repo: &Repository) -> Result<Vec<(PathBuf, Oid)>, Error> {
    let head = repo.head().and_then(|head| head.peel_to_tree()).ok();
    let diff = repo.diff_tree_to_index(head.as_ref(), None, None)?;

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
