//! The Stele module contains the Stele object for interacting with
//! Stelae.

use std::path::PathBuf;

use crate::stelae::types::{dependencies::Dependencies, repositories::Repositories};
use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::fs::read_to_string;

/// Stele
#[derive(Debug, Clone)]
pub struct Stele {
    /// Path to the containing Stelae archive.
    pub archive_path: PathBuf,
    /// Name of the authentication repo (e.g. law).
    pub name: String,
    /// Name of the Stele's directory, also known as Stele's organization (e.g. openlawlibrary).
    pub org: String,
    /// Full path to the Stele's directory.
    pub path: PathBuf,
    /// Stele's repositories (as specified in repositories.json).
    pub repositories: Option<Repositories>,
    /// Indicates whether or not the Stele is the root Stele.
    pub root: bool,
}

impl Stele {
    /// Create a new Stele object
    /// # Errors
    /// Will error if unable to find or parse repositories file at `targets/repositories.json`
    /// # Panics
    /// Will panic if unable to determine the current root Stele.
    #[allow(clippy::unwrap_used, clippy::shadow_reuse)]
    pub fn new(
        archive_path: PathBuf,
        name: Option<String>,
        org: Option<String>,
        path: Option<PathBuf>,
        root: bool,
    ) -> anyhow::Result<Self> {
        let name = name.unwrap_or_else(|| "law".to_owned());
        let org = org.unwrap_or_else(|| {
            path.as_ref()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
        });
        let path = path.unwrap_or_else(|| archive_path.join(&org));
        let mut stele = Self {
            archive_path,
            name,
            org,
            path,
            repositories: None,
            root,
        };
        stele.get_repositories()?;
        Ok(stele)
    }

    /// Get Stele's dependencies.
    /// # Errors
    /// Will error if unable to parse dependencies file from `targets/dependencies.json`
    pub fn get_dependencies(&self) -> anyhow::Result<Option<Dependencies>> {
        let dependencies_path = &self.path.join(PathBuf::from(format!(
            "{}/targets/dependencies.json",
            self.name
        )));
        if let Ok(dependencies_str) = read_to_string(dependencies_path) {
            let dependencies = serde_json::from_str(&dependencies_str)?;
            return Ok(Some(dependencies));
        }
        Ok(None)
    }
    /// Get Stele's repositories.
    /// # Errors
    /// Will error if unable to find or parse repositories file at `targets/repositories.json`
    pub fn get_repositories(&mut self) -> anyhow::Result<Option<Repositories>> {
        let repositories_path = &self.path.join(PathBuf::from(format!(
            "{}/targets/repositories.json",
            self.name
        )));
        if let Ok(repositories_str) = read_to_string(repositories_path) {
            let repositories: Repositories = serde_json::from_str(&repositories_str)?;
            self.repositories = Some(repositories.clone());
            return Ok(Some(repositories));
        }
        Ok(None)
    }
}

///Config object for a Stele
#[derive(Deserialize, Serialize)]
pub struct Config {
    /// Name of the authentication repo (e.g. law).
    pub name: String,
    /// Name of the Stele's directory, also known as Stele's organization (e.g. openlawlibrary).
    pub org: String,
    /// The out-of-band authenticated hash of the Stele.
    pub hash: Option<String>,
}
