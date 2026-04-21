use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod manager;

/// Trait for managing transactional stele.
#[async_trait]
pub trait TxManager {
    /// Create a stele.
    async fn create(&mut self, stele: &str) -> anyhow::Result<Option<i64>>;
    /// Delete a stele and all of its associated data via cascade.
    ///
    /// Requires `PRAGMA foreign_keys = ON` (set at connection time) for the
    /// `ON DELETE CASCADE` constraints to take effect.
    async fn delete(&mut self, stele: &str) -> anyhow::Result<()>;
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
/// Model for a Stele.
pub struct Stele {
    /// Stele identifier in <org>/<name> format.
    /// Example: `org-name/repo-name-law`.
    pub name: String,
}
