use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{any::AnyRow, FromRow, Row};

pub mod manager;

/// Trait for managing transactional authentication commits.
#[async_trait]
pub trait TxManager {
    /// Find all authentication commits.
    async fn find_all(&mut self) -> anyhow::Result<Vec<AuthCommits>>;
    /// Insert a bulk of auth commits.
    async fn insert_bulk(&mut self, auth_commits: Vec<AuthCommits>) -> anyhow::Result<()>;
}

#[derive(Debug, Deserialize, Serialize)]
/// Table used for the commits within the authentication repository.
pub struct AuthCommits {
    /// Unique commit hash of the authentication repository.
    pub commit_hash: String,
    /// Timestamp of the time the commit was created.
    pub timestamp: String,
    /// Foreign key reference to the publication version.
    pub publication_version_id: Option<String>,
}

/// NOTE: current sqlx version does not support `Option` types in `FromRow` trait.
/// For now, we manually implement the `FromRow` for the struct.
/// From version 0.8.0, sqlx has resolved the issue.
impl FromRow<'_, AnyRow> for AuthCommits {
    fn from_row(row: &AnyRow) -> anyhow::Result<Self, sqlx::Error> {
        Ok(Self {
            commit_hash: row.try_get("commit_hash")?,
            timestamp: row.try_get("timestamp")?,
            publication_version_id: row.try_get("publication_version_id").ok(),
        })
    }
}

impl AuthCommits {
    /// Create a new authentication commit.
    #[must_use]
    pub const fn new(
        commit_hash: String,
        timestamp: String,
        publication_version_id: Option<String>,
    ) -> Self {
        Self {
            commit_hash,
            timestamp,
            publication_version_id,
        }
    }
}
