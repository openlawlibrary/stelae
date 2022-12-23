//! The library module contains structs for interacting with a Stele library

use std::path::{Path, PathBuf};

/// given a &Path `path`, return the path to the containing library.
///
/// # Errors
/// Error if the path doesn't exist or isn't inside a Stele library.
pub fn find_library_path(path: &Path) -> anyhow::Result<PathBuf> {
    let abs_path = path.canonicalize()?;
    for working_path in abs_path.ancestors() {
        if working_path.join(".stele").exists() {
            return Ok(working_path.to_owned());
        }
    }
    anyhow::bail!(format!(
        "{} is not inside a Stele Library. Run `stele init` to create a library at this location.",
        abs_path.to_string_lossy()
    ))
}
