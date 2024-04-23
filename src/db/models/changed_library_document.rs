use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Serialize)]
/// Model for library (collection) change events.
pub struct ChangedLibraryDocument {
    /// Foreign key reference to publication name
    pub publication: String,
    /// Foreign key reference to codified date in a publication in %Y-%m-%d format
    pub version: String,
    /// Foreign key reference to stele identifier in <org>/<name> format.
    pub stele: String,
    /// Materialized path to the document
    pub doc_mpath: String,
    /// Change status of the document.
    /// Currently could be 'Element added', 'Element effective', 'Element changed' or 'Element removed'.
    pub status: String,
    /// Materialized path to the library
    pub library_mpath: String,
    /// Url to the library that was changed.
    pub url: String,
}
