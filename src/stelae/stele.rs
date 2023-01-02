//! The Stele module contains the Stele object for interacting with
//! Stelae.

use crate::stelae::archive::Archive;
use serde_derive::{Deserialize, Serialize};

/// Stele
pub struct Stele<'stele> {
    /// Pointer to the containing Archive struct.
    pub archive: &'stele Archive<'stele>,
    /// Fully qualified name of the authentication repo (e.g. openlawlibrary/law).
    pub name: String,
}

impl Stele<'_> {
}

///Config object for a Stele
#[derive(Deserialize, Serialize)]
pub struct Config {
    /// The fully qualified name of the Stele (e.g. openlawlibrary/law)
    pub name: String,
    /// The out-of-band authenticated hash of the Stele.
    pub hash: Option<String>,
}
