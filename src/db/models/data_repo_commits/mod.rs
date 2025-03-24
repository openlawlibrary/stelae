use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod manager;

/// Trait for managing transactional data repo commits.
#[async_trait]
pub trait TxManager {
    /// Find all authentication commits for a given stele.
    async fn find_all_auth_commits_for_stele(
        &mut self,
        stele_id: &str,
    ) -> anyhow::Result<Vec<DataRepoCommits>>;
    /// Insert a bulk of data repo commits.
    async fn insert_bulk(&mut self, data_repo_commits: Vec<DataRepoCommits>) -> anyhow::Result<()>;
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
/// Model for the commits within the data repository.
pub struct DataRepoCommits {
    /// Unique commit hash of the authentication repository.
    pub commit_hash: String,
    /// Either codified date or date on which the commit was built on (build-date).
    pub date: String,
    /// Type of the data repository. E.g. `html`.
    pub repo_type: String,
    /// Foreign key reference to the authentication commit hash.
    pub auth_commit_hash: String,
    /// Timestamp of the authentication commit.
    pub auth_commit_timestamp: String,
    /// Foreign key reference to the publication.
    pub publication_id: String,
    /// Type of commit: 0 for build date, 1 for codified date.
    #[serde(default)]
    pub commit_type: i64,
}

impl DataRepoCommits {
    /// Create a new data commit.
    #[must_use]
    pub const fn new(
        commit_hash: String,
        date: String,
        repo_type: String,
        auth_commit_hash: String,
        auth_commit_timestamp: String,
        publication_id: String,
        commit_type: i64,
    ) -> Self {
        Self {
            commit_hash,
            date,
            repo_type,
            auth_commit_hash,
            auth_commit_timestamp,
            publication_id,
            commit_type,
        }
    }
}
