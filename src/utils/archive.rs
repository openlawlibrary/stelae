//! The archive module contains structs for interacting with a Stele archive

use super::paths::fix_unc_path;
use anyhow::Context;
use std::path::{Path, PathBuf};

/// given a &Path `path`, return the path to the containing archive.
///
/// # Errors
/// Error if the path doesn't exist or isn't inside a Stele archive.
pub fn find_archive_path(path: &Path) -> anyhow::Result<PathBuf> {
    let abs_path = fix_unc_path(&path.canonicalize()?);
    for working_path in abs_path.ancestors() {
        if working_path.join(".stelae").exists() {
            return Ok(working_path.to_owned());
        }
    }
    anyhow::bail!(format!(
        "{} is not inside a Stelae Archive. Run `stelae init` to create a archive at this location.",
        abs_path.to_string_lossy()
    ))
}

/// Get the qualified name as parts of a Stele from the {org}/{name} format.
/// # Errors
/// Will error if the qualified name is not in the {org}/{name} format.
pub fn get_name_parts(qualified_name: &str) -> anyhow::Result<(String, String)> {
    let mut name_parts = qualified_name.split('/');
    let org = name_parts.next().context("No organization specified");
    let name = name_parts.next().context("No name specified");
    Ok((org?.to_owned(), name?.to_owned()))
}

#[cfg(test)]
mod test {
    use crate::utils::archive::get_name_parts;

    #[test]
    fn get_name_parts_when_qualified_name_correct_expect_name_parts() {
        let cut = get_name_parts;
        let actual = cut("stele/test").unwrap();
        let expected = ("stele".to_owned(), "test".to_owned());
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_name_parts_when_qualified_name_incorrect_expect_error() {
        let cut = get_name_parts;
        let actual = cut("test").unwrap_err();
        let expected = "No name specified";
        assert!(
            actual.to_string().contains(expected),
            "\"{actual}\" doesn't contain {expected}"
        );
    }
}
