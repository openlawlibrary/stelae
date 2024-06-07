use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod manager;

/// Trait for managing transactions on publication versions.
#[async_trait]
pub trait TxManager {
    /// Create a new publication version.
    async fn insert_bulk(&mut self, libraries: Vec<Library>) -> anyhow::Result<()>;
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
/// Model for library (collection).
pub struct Library {
    /// Materialized path to the library
    pub mpath: String,
}
