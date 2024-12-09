//! Manager for the data repo commit model.
use async_trait::async_trait;
use sqlx::QueryBuilder;

use crate::db::{models::BATCH_SIZE, DatabaseTransaction};

use super::DataCommits;

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Upsert a bulk of data repository commits into the database.
    ///
    /// # Errors
    /// Errors if the commits cannot be inserted.
    async fn insert_bulk(&mut self, data_commits: Vec<DataCommits>) -> anyhow::Result<()> {
        let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO data_commits ( commit_hash, date, data_repo_type, auth_commit_hash, publication_id ) ");
        for chunk in data_commits.chunks(BATCH_SIZE) {
            query_builder.push_values(chunk, |mut bindings, dc| {
                bindings
                    .push_bind(&dc.commit_hash)
                    .push_bind(&dc.date)
                    .push_bind(&dc.data_repo_type)
                    .push_bind(&dc.auth_commit_hash)
                    .push_bind(&dc.publication_id);
            });
            let query = query_builder.build();
            query.execute(&mut *self.tx).await?;
            query_builder.reset();
        }
        Ok(())
    }
}
