use super::version::Version;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod manager;

/// Trait for managing document changes.
#[async_trait]
pub trait Manager {
    /// All dates on which given document changed.
    async fn find_all_document_versions_by_mpath_and_publication(
        &self,
        mpath: &str,
        publication: &str,
    ) -> anyhow::Result<Vec<Version>>;
}

/// Trait for managing transactional document changes.
#[async_trait]
pub trait TxManager {
    /// Insert a bulk of document changes.
    async fn insert_bulk(&mut self, document_changes: Vec<DocumentChange>) -> anyhow::Result<()>;
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
/// Model for document change events.
pub struct DocumentChange {
    /// A hashed identifier for the document change.
    /// The hash is generated from the `publication_version` id, `doc_mpath`, and `status` (as integer) field.
    pub id: String,
    /// Change status of the document.
    /// Currently could be 'Element added' = 0, 'Element effective' = 1, 'Element changed' = 2 or 'Element removed' = 3.
    pub status: i64,
    /// Optional reason for the change event.
    pub change_reason: Option<String>,
    /// Foreign key reference to the `publication_version` id.
    pub publication_version_id: String,
    /// Materialized path to the document
    pub doc_mpath: String,
}
