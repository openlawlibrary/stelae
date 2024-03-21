use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Serialize)]
/// Model for document change events.
pub struct DocumentChange {
    /// Materialized path to the document
    pub doc_mpath: String,
    /// Change status of the document.
    /// Currently could be 'Element added', 'Element effective', 'Element changed' or 'Element removed'.
    pub status: String,
    /// Url to the document that was changed.
    pub url: String,
    /// Optional reason for the change event.
    pub change_reason: Option<String>,
    /// Foreign key reference to the publication name.
    pub publication: String,
    /// Foreign key reference to codified date in a publication in %Y-%m-%d format
    pub version: String,
    /// Foreign key reference to stele identifier in <org>/<name> format.
    pub stele: String,
    /// Foreign key reference to document id.
    pub doc_id: String,
}
