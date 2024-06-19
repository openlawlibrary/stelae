use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod manager;

/// Trait for managing collection changes.
#[async_trait]
pub trait Manager {
    /// Find one library materialized path by url.
    async fn find_lib_mpath_by_url(&self, url: &str) -> anyhow::Result<String>;
}

/// Trait for managing transactions on publication versions.
#[async_trait]
pub trait TxManager {
    /// Insert bulk libraries.
    async fn insert_bulk(&mut self, libraries: Vec<Library>) -> anyhow::Result<()>;
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
/// Model for library (collection).
pub struct Library {
    /// Materialized path to the collection
    pub mpath: String,
    /// Url to the collection.
    pub url: String,
}

impl Library {
    /// Create a new library.
    #[must_use]
    pub const fn new(mpath: String, url: String) -> Self {
        Self { mpath, url }
    }
}
