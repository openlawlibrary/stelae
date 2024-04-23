//! A Stele's dependencies.
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

/// A map of Stele names to their dependencies.
#[derive(Serialize, Deserialize, Debug)]
pub struct Dependencies {
    /// An inner map of Stele keys to their dependencies.
    pub dependencies: HashMap<String, Dependency>,
}

/// A single dependency as specified in a Stele's `dependencies.json` file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Dependency {
    /// The out-of-band authenticated hash of the Stele.
    #[serde(rename = "out-of-band-authentication")]
    pub out_of_band_authentication: String,
    /// The default branch for a Stele.
    pub branch: String,
}

impl Dependencies {
    /// Get the dependencies names for a given Stele.
    #[must_use]
    pub fn sorted_dependencies_names(&self) -> Vec<String> {
        let mut keys = self.dependencies.keys().cloned().collect::<Vec<String>>();
        keys.sort();
        keys
    }
}
