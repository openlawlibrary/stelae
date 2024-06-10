use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod manager;

/// Trait for managing transactional changed library documents.
#[async_trait]
pub trait TxManager {
    /// Insert bulk of changed library documents.
    async fn insert_bulk(
        &mut self,
        changed_library_document: Vec<ChangedLibraryDocument>,
    ) -> anyhow::Result<()>;
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
/// Model for library (collection) change events.
pub struct ChangedLibraryDocument {
    /// Foreign key reference to `document_change` id.
    pub document_change_id: String,
    /// Materialized path to the library
    pub library_mpath: String,
}
