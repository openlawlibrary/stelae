use super::version::Version;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod manager;

/// Trait for managing collection changes.
#[async_trait]
pub trait Manager {
    /// All dates on which given documents within a collection changed.
    async fn find_all_collection_versions_by_mpath_and_publication(
        &self,
        mpath: &str,
        publication: &str,
    ) -> anyhow::Result<Vec<Version>>;
}

/// Trait for managing transactional collection changes.
#[async_trait]
pub trait TxManager {
    /// Insert a bulk of collection changes.
    async fn insert_bulk(&mut self, library_changes: Vec<LibraryChange>) -> anyhow::Result<()>;
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug)]
/// Model for library (collection) change events.
pub struct LibraryChange {
    /// Foreign key reference to `publication_version` id.
    pub publication_version_id: String,
    /// Change status of the document.
    /// Currently could be 'Element added' = 0, 'Element effective' = 1, 'Element changed' = 2 or 'Element removed' = 3.
    pub status: i64,
    /// Materialized path to the library
    pub library_mpath: String,
}

impl LibraryChange {
    /// Create a new library change.
    #[must_use]
    pub const fn new(publication_version_id: String, status: i64, library_mpath: String) -> Self {
        Self {
            publication_version_id,
            status,
            library_mpath,
        }
    }
}
