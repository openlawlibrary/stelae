//! The Stele module contains the Stele object for interacting with
//! Stelae.

use std::path::PathBuf;

use crate::stelae::types::{dependencies::Dependencies, repositories::Repositories};
use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::fs::read_to_string;

/// Stele
#[derive(Debug, Clone, Default)]
pub struct Stele {
    /// Path to the containing Stelae archive.
    pub archive_path: PathBuf,
    /// Fully qualified name of the authentication repo (e.g. openlawlibrary/law).
    pub name: String,
    ///Full path to the Stele's directory.
    pub path: PathBuf,
}

impl Stele {
    /// Get Stele's dependencies.
    /// # Errors
    /// Will error if unable to parse dependencies file from `targets/dependencies.json`
    pub fn get_dependencies(&self) -> anyhow::Result<Option<Dependencies>> {
        let dependencies_path = &self.archive_path.join(PathBuf::from(format!(
            "{}/targets/dependencies.json",
            self.name
        )));
        match read_to_string(dependencies_path) {
            Ok(dependencies_str) => {
                let dependencies = serde_json::from_str(&dependencies_str)?;
                Ok(Some(dependencies))
            }
            Err(_) => Ok(None),
        }
    }
    /// Get Stele's repositories.
    /// # Errors
    /// Will error if unable to find or parse repositories file at `targets/repositories.json`
    pub fn get_repositories(&self) -> anyhow::Result<Repositories> {
        let repositories_path = &self.archive_path.join(PathBuf::from(format!(
            "{}/targets/repositories.json",
            self.name
        )));
        let repositories_str = read_to_string(repositories_path)?;
        let repositories = serde_json::from_str(&repositories_str)?;
        Ok(repositories)
    }
}

///Config object for a Stele
#[derive(Deserialize, Serialize)]
pub struct Config {
    /// The fully qualified name of the Stele (e.g. openlawlibrary/law)
    pub name: String,
    /// The out-of-band authenticated hash of the Stele.
    pub hash: Option<String>,
}
