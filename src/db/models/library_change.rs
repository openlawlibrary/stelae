use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::version::Version;

/// Trait for managing collection changes.
#[async_trait]
pub trait Manager {
    /// Find one library materialized path by url.
    async fn find_lib_mpath_by_url(&self, url: &str) -> anyhow::Result<Option<String>>;
    /// All dates on which given documents within a collection changed.
    async fn find_all_collection_versions_by_mpath_and_publication(
        &self,
        mpath: &str,
        publication: &str,
    ) -> anyhow::Result<Vec<Version>>;
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug)]
/// Model for library (collection) change events.
pub struct LibraryChange {
    /// Foreign key reference to publication name
    pub publication: String,
    /// Foreign key reference to codified date in a publication in %Y-%m-%d format
    pub version: String,
    /// Foreign key reference to stele identifier in <org>/<name> format.
    pub stele: String,
    /// Change status of the document.
    /// Currently could be 'Element added', 'Element effective', 'Element changed' or 'Element removed'.
    pub status: String,
    /// Url to the library that was changed.
    pub url: String,
    /// Materialized path to the library
    pub library_mpath: String,
}
