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
}

impl Stele {
    /// Create a new Stele object
    /// # Errors
    /// Will error if unable to find or parse repositories file at `targets/repositories.json`
    pub fn new(
        archive_path: PathBuf,
        name: String,
        org: String,
        path: PathBuf,
    ) -> anyhow::Result<Self> {
        let mut stele = Self {
            archive_path,
            name,
            org,
            path,
            repositories: None,
        };
        stele.get_repositories()?;
        Ok(stele)
    }

    /// Create a new Stele object given a file path and an archive.
    /// # Errors
    /// Will if unable to find or parse repositories file at `targets/repositories.json`.
    /// # Panics
    /// Will panic if unable to determine the Stele's organization.
    pub fn new_individual(archive_path: PathBuf, path: PathBuf) -> anyhow::Result<Self> {
        let org = path.file_name().unwrap().to_str().unwrap();
        let mut stele = Self {
            org: org.to_owned(),
            name: "law".to_owned(),
            path,
            archive_path,
            repositories: None,
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
            let mut repositories: Repositories = serde_json::from_str(&repositories_str)?;
            dbg!(&repositories);
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
