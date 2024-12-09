use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod manager;

/// Trait for managing transactional data repo commits.
#[async_trait]
pub trait TxManager {
    /// Insert a bulk of data repo commits.
    async fn insert_bulk(&mut self, data_commits: Vec<DataCommits>) -> anyhow::Result<()>;
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
/// Model for the commits within the data repository.
pub struct DataCommits {
    /// Unique commit hash of the authentication repository.
    pub commit_hash: String,
    /// Either codified date or date on which the commit was built on (build-date).
    pub date: String,
    /// Type of the data repository. E.g. `html`.
    pub data_repo_type: String,
    /// Foreign key reference to the authentication commit hash.
    pub auth_commit_hash: String,
    /// Foreign key reference to the publication.
    pub publication_id: String,
}

impl DataCommits {
    /// Create a new data commit.
    #[must_use]
    pub const fn new(
        commit_hash: String,
        date: String,
        data_repo_type: String,
        auth_commit_hash: String,
        publication_id: String,
    ) -> Self {
        Self {
            commit_hash,
            date,
            data_repo_type,
            auth_commit_hash,
            publication_id,
        }
    }
}
