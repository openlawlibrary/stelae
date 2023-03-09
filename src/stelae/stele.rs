//! The Stele module contains the Stele object for interacting with
//! Stelae.

use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

/// Stele
#[derive(Debug, Clone)]
pub struct Stele {
    /// Path to the containing Stelae archive.
    pub archive_path: PathBuf,
    /// Fully qualified name of the authentication repo (e.g. openlawlibrary/law).
    pub name: String,
}

impl Stele {
}

///Config object for a Stele
#[derive(Deserialize, Serialize)]
pub struct Config {
    /// The fully qualified name of the Stele (e.g. openlawlibrary/law)
    pub name: String,
    /// The out-of-band authenticated hash of the Stele.
    pub hash: Option<String>,
}
