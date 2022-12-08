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
    Err(anyhow::anyhow!(format!(
        "{} is not inside a Stele Library. Run `stele init` to create a library at this location.",
        abs_path.to_string_lossy()
    )))
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::string_slice)]
#[allow(clippy::indexing_slicing)]
#[cfg(test)]
mod test {
    use crate::utils::library::find_library_path;
    use std::env::current_exe;
    use std::path::PathBuf;

    fn get_test_library_path() -> PathBuf {
        let mut library_path = current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        library_path.push("test");
        library_path.push("library");
        library_path.canonicalize().unwrap()
    }

    #[test]
    fn test_find_library_path_when_at_library_expect_path() {
        let library_path = get_test_library_path();
        let actual = find_library_path(&library_path).unwrap();
        let expected = library_path;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_find_library_path_when_in_library_expect_library_path() {
        let library_path = get_test_library_path();
        let cwd = library_path.join("test");
        let actual = find_library_path(&cwd).unwrap();
        let expected = library_path;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_find_library_path_when_nonexistant_path_expect_error() {
        let library_path = get_test_library_path();
        let cwd = library_path.join("does_not_exist");
        let actual_err = find_library_path(&cwd).unwrap_err();
        let actual = format!("{}", actual_err);
        let expected = "(os error 2)";
        assert_eq!(&actual[actual.len() - 12..], expected);
    }

    #[test]
    fn test_find_library_path_when_not_in_library_expect_error() {
        let library_path = get_test_library_path();
        let cwd = library_path.parent().unwrap();
        let actual_err = find_library_path(cwd).unwrap_err();
        let actual = format!("{}", actual_err);
        let expected =
            "is not inside a Stele Library. Run `stele init` to create a library at this location.";
        assert_eq!(&actual[actual.len() - 85..], expected);
    }
}
