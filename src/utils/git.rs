//! The git module contains structs for interacting with git repositories
//! in the Stele Library.
use git2::{Error, Repository};
use std::fmt;

/// Represents a git repository within an oll library. includes helpers for
/// for interacting with the Git Repo.
/// Expects a path to the library, as well as the repo's namespace and name.
pub struct Repo {
    /// Path to the library
    lib_path: String,
    /// Repo namespace
    namespace: String,
    /// Repo name
    name: String,
    /// git2 repository pointing to the repo in the library.
    repo: Repository,
}

impl fmt::Debug for Repo {
    #[inline]
    #[allow(clippy::implicit_return)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Repo for {}/{} in the library at {}",
            self.namespace, self.name, self.lib_path
        )
    }
}

impl Repo {
    /// Create a new Repo object with helpers for interacting with a Git Repo.
    /// Expects a path to the library, as well as the repo's namespace and name.
    ///
    /// # Errors
    ///
    /// Will return `Err` if git repository does not exist at `{namespace}/{name}`
    /// in library, or if there is something wrong with the git repository.

    #[allow(clippy::implicit_return)]
    #[inline]
    pub fn new(lib_path: &str, namespace: &str, name: &str) -> Result<Self, Error> {
        let repo_path = format!("{lib_path}/{namespace}/{name}");
        Ok(Self {
            lib_path: String::from(lib_path),
            namespace: String::from(namespace),
            name: String::from(name),
            repo: Repository::open(repo_path)?,
        })
    }

    /// Returns bytes of blob found in the commit `commitish` at path `path`
    /// if a blob is not found at path, it will try adding ".html", "index.html,
    /// and "/index.html".
    /// Example usage:
    ///
    //// let content: Vec<u8> = repo.get_bytes_at_path(
    ////    "0f2f1ef9fa213dcf83e269bc832ab63435cbd4b1",
    ////    "us/ca/cities/san-mateo"
    //// );
    ///
    /// # Errors
    ///
    /// Will return `Err` if `commitish` does not exist in repo, if a blob does
    /// not exist in commit at `path`, or if there is a problem with reading repo.
    #[allow(clippy::implicit_return)]
    #[inline]
    pub fn get_bytes_at_path(&self, commitish: &str, path: &str) -> anyhow::Result<Vec<u8>> {
        let base_revision = format!("{commitish}:{path}");
        for postfix in ["", "/index.html", ".html", "index.html"] {
            match self
                .repo
                .revparse_single(&format!("{base_revision}{postfix}"))
            {
                Ok(obj) => {
                    let blob = match obj.into_blob() {
                        Ok(blob) => blob,
                        Err(_) => continue,
                    };
                    return Ok(blob.content().to_owned());
                }
                Err(_) => continue,
            }
        }
        Err(anyhow::anyhow!("Doesn't exist"))
    }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::string_slice)]
#[allow(clippy::indexing_slicing)]
#[cfg(test)]
mod tests {
    use crate::utils::git::Repo;
    use std::env::current_exe;
    use std::fs::create_dir_all;
    use std::path::PathBuf;
    use std::sync::Once;

    static INIT: Once = Once::new();

    pub fn initialize() {
        INIT.call_once(|| {
            let repo_path = get_test_library_path().join(PathBuf::from("test/law-html"));
            let heads_path = repo_path.join(PathBuf::from("refs/heads"));
            create_dir_all(heads_path).unwrap();
            let tags_path = repo_path.join(PathBuf::from("refs/tags"));
            create_dir_all(tags_path).unwrap();
        });
    }

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
        library_path
    }

    #[test]
    fn test_get_bytes_at_path_when_empty_path_expect_index_html() {
        initialize();
        let test_library_path = get_test_library_path();
        let repo = Repo::new(test_library_path.to_str().unwrap(), "test", "law-html").unwrap();
        let actual = repo
            .get_bytes_at_path("ed782e08d119a580baa3067e2ea5df06f3d1cd05", "")
            .unwrap();
        let expected = "<!DOCTYPE html>";
        assert_eq!(
            &core::str::from_utf8(actual.as_slice()).unwrap()[..15],
            expected
        );
    }

    #[test]
    fn test_get_bytes_at_path_when_full_path_expect_data() {
        initialize();
        let test_library_path = get_test_library_path();
        let repo = Repo::new(test_library_path.to_str().unwrap(), "test", "law-html").unwrap();
        let actual = repo
            .get_bytes_at_path("ed782e08d119a580baa3067e2ea5df06f3d1cd05", "a/b/c.html")
            .unwrap();
        let expected = "<!DOCTYPE html>";
        assert_eq!(
            &std::str::from_utf8(actual.as_slice()).unwrap()[..15],
            expected
        );
    }

    #[test]
    fn test_get_bytes_at_path_when_omit_html_expect_data() {
        initialize();
        let test_library_path = get_test_library_path();
        let repo = Repo::new(test_library_path.to_str().unwrap(), "test", "law-html").unwrap();
        let actual = repo
            .get_bytes_at_path("ed782e08d119a580baa3067e2ea5df06f3d1cd05", "a/b/c")
            .unwrap();
        let expected = "<!DOCTYPE html>";
        assert_eq!(
            &std::str::from_utf8(actual.as_slice()).unwrap()[..15],
            expected
        );
    }

    #[test]
    fn test_get_bytes_at_path_when_omit_index_expect_data() {
        initialize();
        let test_library_path = get_test_library_path();
        let repo = Repo::new(test_library_path.to_str().unwrap(), "test", "law-html").unwrap();
        let actual = repo
            .get_bytes_at_path("ed782e08d119a580baa3067e2ea5df06f3d1cd05", "a/b/d")
            .unwrap();
        let expected = "<!DOCTYPE html>";
        assert_eq!(
            &std::str::from_utf8(actual.as_slice()).unwrap()[..15],
            expected
        );
    }

    #[test]
    fn test_get_bytes_at_path_when_invalid_repo_namespace_expect_error() {
        initialize();
        let test_library_path = get_test_library_path();
        let actual = Repo::new(test_library_path.to_str().unwrap(), "xxx", "law-html").unwrap_err();
        let expected = "failed to resolve path";
        assert_eq!(&format!("{}", actual)[..22], expected);
    }

    #[test]
    fn test_get_bytes_at_path_when_invalid_repo_name_expect_error() {
        initialize();
        let test_library_path = get_test_library_path();
        let actual = Repo::new(test_library_path.to_str().unwrap(), "test", "xxx").unwrap_err();
        let expected = "failed to resolve path";
        assert_eq!(&format!("{}", actual)[..22], expected);
    }

    #[test]
    fn test_get_bytes_at_path_when_invalid_path_expect_error() {
        initialize();
        let test_library_path = get_test_library_path();
        let repo = Repo::new(test_library_path.to_str().unwrap(), "test", "law-html").unwrap();
        let actual = repo
            .get_bytes_at_path("ed782e08d119a580baa3067e2ea5df06f3d1cd05", "a/b/x")
            .unwrap_err();
        let expected = "Doesn't exist";
        assert_eq!(format!("{}", actual), expected);
    }
}
