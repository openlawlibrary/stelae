//! The archive module contains structs for interacting with a Stele archive

use std::path::{Path, PathBuf};

use anyhow::Context;

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

/// Get the qualified name as parts of a Stele from the {org}/{name} format.
/// # Examples
/// ```
/// use stelae::utils::archive::get_name_parts;
/// let (org, name) = get_name_parts("law/stele");
/// assert_eq!(org, "law");
/// assert_eq!(name, "stele");
/// ```
/// # Errors
/// Will error if the qualified name is not in the {org}/{name} format.
pub fn get_name_parts(qualified_name: &str) -> anyhow::Result<(String, String)> {
    let mut name_parts = qualified_name.split('/');
    let org = name_parts.next().context("No organization specified");
    let name = name_parts.next().context("No name specified");
    Ok((org?.to_owned(), name?.to_owned()))
}