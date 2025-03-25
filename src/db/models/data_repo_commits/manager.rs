//! Manager for the data repo commit model.
use async_trait::async_trait;
use sqlx::QueryBuilder;

use crate::db::{models::BATCH_SIZE, DatabaseTransaction};

use super::DataRepoCommits;

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Find all authentication commits for a given stele.
    ///
    /// # Errors
    /// Errors if the commits cannot be found.
    async fn find_all_auth_commits_for_stele(
        &mut self,
        stele_id: &str,
    ) -> anyhow::Result<Vec<DataRepoCommits>> {
        let query = "
            SELECT dc.*
            FROM data_repo_commits dc
            LEFT JOIN publication p ON dc.publication_id = p.id
            LEFT JOIN stele s ON p.stele = s.name
            WHERE s.name = $1
        ";
        let data_repo_commits = sqlx::query_as::<_, DataRepoCommits>(query)
            .bind(stele_id)
            .fetch_all(&mut *self.tx)
            .await?;
        Ok(data_repo_commits)
    }
    /// Upsert a bulk of data repository commits into the database.
    ///
    /// # Errors
    /// Errors if the commits cannot be inserted.
    async fn insert_bulk(&mut self, data_repo_commits: Vec<DataRepoCommits>) -> anyhow::Result<()> {
        let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO data_repo_commits ( commit_hash, codified_date, build_date, repo_type, auth_commit_hash, auth_commit_timestamp, publication_id ) ");
        for chunk in data_repo_commits.chunks(BATCH_SIZE) {
            query_builder.push_values(chunk, |mut bindings, dc| {
                bindings
                    .push_bind(&dc.commit_hash)
                    .push_bind(&dc.codified_date)
                    .push_bind(&dc.build_date)
                    .push_bind(&dc.repo_type)
                    .push_bind(&dc.auth_commit_hash)
                    .push_bind(&dc.auth_commit_timestamp)
                    .push_bind(&dc.publication_id);
            });
            let query = query_builder.build();
            query.execute(&mut *self.tx).await?;
            query_builder.reset();
        }
        Ok(())
    }
}
