use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Serialize)]
/// Model for document change events.
pub struct DocumentChange {
    /// Database id.
    pub id: i32,
    /// Materialized path to the document
    pub doc_mpath: String,
    /// Change status of the document.
    /// Currently could be 'Element added', 'Element effective', 'Element changed' or 'Element removed'.
    pub status: String,
    /// Url to the document that was changed.
    pub url: String,
    /// Optional reason for the change event.
    pub change_reason: Option<String>,
    /// Foreign key reference to publication_version id.
    pub publication_version_id: i32,
    /// Foreign key reference to document id.
    pub document_id: i32,
}
