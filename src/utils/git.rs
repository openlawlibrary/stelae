//! The git module contains structs for interacting with git repositories
//! in the Stelae Archive.
use crate::utils::paths::clean_path;
use anyhow::Context as _;
use git2::{Commit, Repository, Sort};
use std::{
    fmt,
    path::{Path, PathBuf},
};

/// This is the first step towards having custom errors
pub const GIT_REQUEST_NOT_FOUND: &str = "Git object doesn't exist";

/// Represents a git repository within an oll archive. includes helpers for
/// for interacting with the Git Repo.
/// Expects a path to the archive, as well as the repo's organization and name.
pub struct Repo {
    /// Path to the archive
    pub archive_path: String,
    /// Path to the Stele
    pub path: PathBuf,
    /// Repo organization
    pub org: String,
    /// Repo name
    pub name: String,
    /// git2 repository pointing to the repo in the archive.
    pub repo: Repository,
}

/// Represents a git blob returned from the archive on disk
#[derive(Debug)]
pub struct Blob {
    /// The actual content of the git blob
    pub content: Vec<u8>,
    /// Path to the blob
    pub path: String,
}

impl fmt::Debug for Repo {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "Repo for {}/{} in the archive at {}",
            self.org, self.name, self.archive_path
        )
    }
}

#[expect(
    clippy::missing_trait_methods,
    clippy::unwrap_used,
    reason = "Expect to have git repo on disk"
)]
impl Clone for Repo {
    fn clone(&self) -> Self {
        Self {
            archive_path: self.archive_path.clone(),
            org: self.org.clone(),
            name: self.name.clone(),
            path: self.path.clone(),
            repo: Repository::open(self.path.clone()).unwrap(),
        }
    }
}

impl Repo {
    /// Find something like `abc123:/path/to/something.txt` in the Git repo
    fn find(&self, query: &str) -> anyhow::Result<Vec<u8>> {
        tracing::trace!(query, "Git reverse parse search");
        let obj = self.repo.revparse_single(query)?;
        let blob = obj.as_blob().context("Couldn't cast Git object to blob")?;
        Ok(blob.content().to_owned())
    }

    /// Do the work of looking for the requested Git object.
    ///
    ///
    /// # Errors
    /// Will error if the Repo couldn't be found, or if there was a problem with the Git object.
    pub fn find_blob(
        archive_path: &Path,
        namespace: &str,
        name: &str,
        remainder: &str,
        commitish: &str,
    ) -> anyhow::Result<Blob> {
        let repo = Self::new(archive_path, namespace, name)?;
        let blob_path = clean_path(remainder);
        let blob = repo.get_bytes_at_path(commitish, &blob_path)?;
        Ok(blob)
    }
    /// Create a new Repo object with helpers for interacting with a Git Repo.
    /// Expects a path to the archive, as well as the repo's org and name.
    ///
    /// # Errors
    ///
    /// Will return `Err` if git repository does not exist at `{org}/{name}`
    /// in archive, or if there is something wrong with the git repository.
    pub fn new(archive_path: &Path, org: &str, name: &str) -> anyhow::Result<Self> {
        let archive_path_str = archive_path.to_string_lossy();
        tracing::trace!(org, name, "Creating new Repo at {archive_path_str}");
        let repo_path = format!("{archive_path_str}/{org}/{name}");
        Ok(Self {
            archive_path: archive_path_str.into(),
            org: org.into(),
            name: name.into(),
            path: PathBuf::from(repo_path.clone()),
            repo: Repository::open(repo_path)?,
        })
    }

    /// Create a new Repo object from a full path to the repo in the archive.
    ///
    /// # Errors
    /// Will return `Err` if the path does not contain at least org and name,
    /// or if git repository does not exist at `{org}/{name}` in archive, or
    /// if there is something wrong with the git repository.
    pub fn from_path(path: &Path) -> anyhow::Result<Self> {
        let components: Vec<&str> = path
            .components()
            .filter_map(|component| component.as_os_str().to_str())
            .collect();
        if components.len() < 2 {
            anyhow::bail!("Path must contain at least org and name");
        }
        let name = (*components
            .last()
            .ok_or_else(|| anyhow::anyhow!("Missing repo name"))?)
        .to_owned();
        let org = (*components
            .get(components.len() - 2)
            .ok_or_else(|| anyhow::anyhow!("Missing repo org"))?)
        .to_owned();
        let archive_path_slice = components.get(..components.len() - 2).ok_or_else(|| {
            anyhow::anyhow!("Path does not contain enough components for archive_path")
        })?;
        let archive_path = archive_path_slice.iter().collect::<PathBuf>();
        Self::new(&archive_path, &org, &name)
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
    pub fn get_bytes_at_path(&self, commitish: &str, path: &str) -> anyhow::Result<Blob> {
        let base_revision = format!("{commitish}:{path}");
        for postfix in ["", "/index.html", ".html", "index.html"] {
            let query = format!("{base_revision}{postfix}");
            let blob = self.find(&query);
            if let Ok(content) = blob {
                let filepath = format!("{path}{postfix}");
                tracing::trace!(query, "Found Git object");
                return Ok(Blob {
                    content,
                    path: filepath,
                });
            }
        }
        tracing::debug!(base_revision, "Couldn't find requested Git object");
        anyhow::bail!(GIT_REQUEST_NOT_FOUND)
    }

    /// Instantiates a git revwalk from the beginning of the repository.
    /// Return an iterator over the commits.
    ///
    /// # Errors
    /// Will error if the revwalk could not be instantiated
    pub fn iter_commits(&self) -> anyhow::Result<impl Iterator<Item = Commit>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)?;
        revwalk.push_head()?;
        Ok(revwalk
            .filter_map(|found_oid| {
                let oid = found_oid.ok()?;
                self.repo.find_commit(oid).ok()
            })
            .collect::<Vec<Commit>>()
            .into_iter())
    }
}
