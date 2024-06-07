use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod manager;

/// Trait for managing transactional documents.
#[async_trait]
pub trait TxManager {
    /// Create a new publication version.
    async fn create(&mut self, doc_id: &str) -> anyhow::Result<Option<i64>>;
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
/// Model for documents.
pub struct Document {
    /// Unique document identifier.
    pub doc_id: String,
}
