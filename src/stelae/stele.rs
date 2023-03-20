//! The Stele module contains the Stele object for interacting with
//! Stelae.

use std::path::PathBuf;

use crate::stelae::dependency::Dependencies;
use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::fs::read_to_string;

/// Stele
#[derive(Debug, Clone)]
pub struct Stele {
    /// Path to the containing Stelae archive.
    pub archive_path: PathBuf,
    /// Fully qualified name of the authentication repo (e.g. openlawlibrary/law).
    pub name: String,
}

impl Stele {
    /// Get the Stele's dependencies.
    /// # Errors
    /// Will error if unable to find or parse dependencies file at `targets/dependencies.json`
    pub fn get_dependencies(&self) -> anyhow::Result<Dependencies> {
        let dependencies_path = &self.archive_path.join(PathBuf::from(format!(
            "{}/targets/dependencies.json",
            self.name
        )));
        let dependencies_str = read_to_string(dependencies_path)?;
        let dependencies = serde_json::from_str(&dependencies_str)?;
        Ok(dependencies)
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
