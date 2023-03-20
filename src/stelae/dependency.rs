//! A Stele's dependencies.
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

/// A map of Stele names to their dependencies.
#[derive(Serialize, Deserialize, Debug)]
pub struct Dependencies {
    /// An inner map of Stele keys to their dependencies.
    dependencies: HashMap<String, Dependency>,
}

/// A single dependency as specified in a Stele's `dependencies.json` file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Dependency {
    /// The out-of-band authenticated hash of the Stele.
    #[serde(rename = "out-of-band-authentication")]
    out_of_band_authentication: String,
    /// The default branch for a Stele.
    branch: String,
}
