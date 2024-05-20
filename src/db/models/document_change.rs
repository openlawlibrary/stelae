use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::version::Version;

/// Trait for managing document changes.
#[async_trait]
pub trait Manager {
    /// Find one document materialized path by url.
    async fn find_doc_mpath_by_url(&self, url: &str) -> anyhow::Result<String>;
    /// All dates on which given document changed.
    async fn find_all_document_versions_by_mpath_and_publication(
        &self,
        mpath: &str,
        publication: &str,
    ) -> anyhow::Result<Vec<Version>>;
}

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
