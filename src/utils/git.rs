//! The git module contains structs for interacting with git repositories
//! in the Stele Library.
use git2::{Error, Repository};
use std::{fmt, path::Path};

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

    pub fn new(lib_path: &Path, namespace: &str, name: &str) -> Result<Self, Error> {
        let lib_path_str = lib_path.to_string_lossy();
        let repo_path = format!("{lib_path_str}/{namespace}/{name}");
        Ok(Self {
            lib_path: String::from(lib_path_str),
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
