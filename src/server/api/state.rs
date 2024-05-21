//! Centralized state management for the Actix web server
use std::{fmt, path::PathBuf};

use crate::{
    db,
    stelae::{archive::Archive, stele::Stele, types::repositories::Repository},
    utils::{archive::get_name_parts, git},
};
use git2::Repository as GitRepository;

/// Global, read-only state
pub trait Global {
    /// Fully initialized Stelae archive
    fn archive(&self) -> &Archive;
    /// Database connection
    fn db(&self) -> &db::DatabaseConnection;
}

/// Application state
#[derive(Debug, Clone)]
pub struct App {
    /// Fully initialized Stelae archive
    pub archive: Archive,
    /// Database connection
    pub db: db::DatabaseConnection,
}

impl Global for App {
    fn archive(&self) -> &Archive {
        &self.archive
    }

    fn db(&self) -> &db::DatabaseConnection {
        &self.db
    }
}

/// Repository to serve
pub struct RepoData {
    /// git2 wrapper repository pointing to the repo in the archive.
    pub repo: git::Repo,
    ///Latest or historical
    pub serve: String,
}

/// Shared, read-only app state
pub struct Shared {
    /// Repository to fall back to if the current one is not found
    pub fallback: Option<RepoData>,
}

impl fmt::Debug for RepoData {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "Repo for {} in the archive at {}",
            self.repo.name,
            self.repo.path.display()
        )
    }
}

impl fmt::Debug for Shared {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let fb = &self.fallback;
        match fb.as_ref() {
            Some(fallback) => write!(
                formatter,
                "Repo for {} in the archive at {}",
                fallback.repo.name,
                fallback.repo.path.display()
            ),
            None => write!(formatter, "No fallback repo"),
        }
    }
}

#[allow(clippy::missing_trait_methods)]
impl Clone for RepoData {
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            serve: self.serve.clone(),
        }
    }
}

#[allow(clippy::missing_trait_methods)]
impl Clone for Shared {
    fn clone(&self) -> Self {
        Self {
            fallback: self.fallback.clone(),
        }
    }
}

/// Initialize the data repository used in the Actix route
/// Each Actix route has its own data repository
///
/// # Errors
/// Will error if unable to initialize the data repository
pub fn init_repo(repo: &Repository, stele: &Stele) -> anyhow::Result<RepoData> {
    let custom = &repo.custom;
    let (org, name) = get_name_parts(&repo.name)?;
    let mut repo_path = stele.archive_path.to_string_lossy().into_owned();
    repo_path = format!("{repo_path}/{org}/{name}");
    Ok(RepoData {
        repo: git::Repo {
            archive_path: stele.archive_path.to_string_lossy().to_string(),
            path: PathBuf::from(&repo_path),
            org,
            name,
            repo: GitRepository::open(&repo_path)?,
        },
        serve: custom.serve.clone(),
    })
}

/// Initialize the shared application state
/// Currently shared application state consists of:
///     - fallback: used as a data repository to resolve data when no other url matches the request
/// # Returns
/// Returns a `SharedState` object
/// # Errors
/// Will error if unable to open the git repo for the fallback data repository
pub fn init_shared(stele: &Stele) -> anyhow::Result<Shared> {
    let fallback = stele
        .get_fallback_repo()
        .map(|repo| {
            let (org, name) = get_name_parts(&repo.name)?;
            Ok::<RepoData, anyhow::Error>(RepoData {
                repo: git::Repo::new(&stele.archive_path, &org, &name)?,
                serve: repo.custom.serve.clone(),
            })
        })
        .transpose()?;
    Ok(Shared { fallback })
}
