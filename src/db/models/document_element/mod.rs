use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod manager;

/// Trait for managing document elements.
#[async_trait]
pub trait Manager {
    /// Find one document materialized path by url.
    async fn find_doc_mpath_by_url(&self, url: &str) -> anyhow::Result<String>;
}

/// Trait for managing transactional document elements.
#[async_trait]
pub trait TxManager {
    /// Insert a bulk of document elements.
    async fn insert_bulk(&mut self, document_elements: Vec<DocumentElement>) -> anyhow::Result<()>;
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
/// Model for document elements.
pub struct DocumentElement {
    /// Materialized path to the document
    pub doc_mpath: String,
    /// Url to the document that was changed.
    pub url: String,
    /// Unique document identifier.
    pub doc_id: String,
}

impl DocumentElement {
    /// Create a new document element.
    #[must_use]
    pub const fn new(doc_mpath: String, url: String, doc_id: String) -> Self {
        Self {
            doc_mpath,
            url,
            doc_id,
        }
    }
}
