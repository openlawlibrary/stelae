use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod manager;
/// Trait for managing versions.
#[async_trait]
pub trait TxManager {
    /// Create a new version.
    async fn create(&mut self, codified_date: &str) -> anyhow::Result<Option<i64>>;
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug, Eq, PartialEq)]
/// Model for a version.
pub struct Version {
    /// Significant codified date of any publication.
    /// Used in the form %YYYY-%MM-%DD.
    pub codified_date: String,
}
