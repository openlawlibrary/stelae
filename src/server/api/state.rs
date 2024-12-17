//! Centralized state management for the Actix web server
use std::{fmt, path::PathBuf};

use crate::{
    db,
    stelae::{archive::Archive, stele::Stele, types::repositories::Repository},
    utils::archive::get_name_parts,
};

/// Global, read-only state
pub trait Global {
    /// Fully initialized Stelae archive
    fn archive(&self) -> &Archive;
    /// Database connection
    fn db(&self) -> &db::DatabaseConnection;
    /// path to the Stelae archive
    fn archive_path(&self) -> &PathBuf;
}

/// Application state
#[derive(Debug, Clone)]
pub struct App {
    /// Fully initialized Stelae archive
    pub archive: Archive,
    /// Database connection
    pub db: db::DatabaseConnection,
    /// path to the Stelae archive
    pub archive_path: PathBuf,
}

impl Global for App {
    fn archive(&self) -> &Archive {
        &self.archive
    }

    fn db(&self) -> &db::DatabaseConnection {
        &self.db
    }

    fn archive_path(&self) -> &PathBuf {
        &self.archive_path
    }
}

/// Repository to serve
pub struct RepoData {
    /// Path to the archive
    pub archive_path: PathBuf,
    /// Path to the Stele
    pub path: PathBuf,
    /// Repo organization
    pub org: String,
    /// Repo name
    pub name: String,
    // /// path to the git repository
    // pub repo_path: PathBuf;
    ///Latest or historical
    pub serve: String,
}

impl RepoData {
    /// Create a new Repo state object
    #[must_use]
    pub fn new(archive_path: &str, org: &str, name: &str, serve: &str) -> Self {
        let mut repo_path = archive_path.to_owned();
        repo_path = format!("{repo_path}/{org}/{name}");
        Self {
            archive_path: PathBuf::from(archive_path),
            path: PathBuf::from(&repo_path),
            org: org.to_owned(),
            name: name.to_owned(),
            serve: serve.to_owned(),
        }
    }
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
            self.name,
            self.path.display()
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
                fallback.name,
                fallback.path.display()
            ),
            None => write!(formatter, "No fallback repo"),
        }
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Use implicit trait implementation"
)]
impl Clone for RepoData {
    fn clone(&self) -> Self {
        Self {
            archive_path: self.archive_path.clone(),
            path: self.path.clone(),
            org: self.org.clone(),
            name: self.name.clone(),
            serve: self.serve.clone(),
        }
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Use implicit trait implementation"
)]
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
    Ok(RepoData::new(
        &stele.archive_path.to_string_lossy(),
        &org,
        &name,
        &custom.serve,
    ))
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
            Ok::<RepoData, anyhow::Error>(RepoData::new(
                &stele.archive_path.to_string_lossy(),
                &org,
                &name,
                &repo.custom.serve,
            ))
        })
        .transpose()?;
    Ok(Shared { fallback })
}
