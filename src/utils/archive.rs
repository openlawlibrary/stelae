//! The archive module contains structs for interacting with a Stele archive

use std::path::{Path, PathBuf};

/// given a &Path `path`, return the path to the containing archive.
///
/// # Errors
/// Error if the path doesn't exist or isn't inside a Stele archive.
pub fn find_archive_path(path: &Path) -> anyhow::Result<PathBuf> {
    let abs_path = path.canonicalize()?;
    for working_path in abs_path.ancestors() {
        if working_path.join(".stelae").exists() {
            return Ok(working_path.to_owned());
        }
    }
    anyhow::bail!(format!(
        "{} is not inside a Stele Archive. Run `stelae init` to create a archive at this location.",
        abs_path.to_string_lossy()
    ))
}
