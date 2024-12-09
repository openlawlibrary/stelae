//! Manager for the auth commit model.

use async_trait::async_trait;
use sqlx::QueryBuilder;

use crate::db::{models::BATCH_SIZE, DatabaseTransaction};

use super::AuthCommits;

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Find all authentication commits.
    ///
    /// # Errors
    /// Errors if the commits cannot be found.
    async fn find_all(&mut self) -> anyhow::Result<Vec<AuthCommits>> {
        let statement = "SELECT * FROM auth_commits";
        let rows = sqlx::query_as::<_, AuthCommits>(statement)
            .fetch_all(&mut *self.tx)
            .await?;
        Ok(rows)
    }
    /// Upsert a bulk of authentication commits into the database.
    ///
    /// # Errors
    /// Errors if the commits cannot be inserted.
    async fn insert_bulk(&mut self, auth_commits: Vec<AuthCommits>) -> anyhow::Result<()> {
        let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO auth_commits ( commit_hash, timestamp, publication_version_id ) ");
        for chunk in auth_commits.chunks(BATCH_SIZE) {
            query_builder.push_values(chunk, |mut bindings, ac| {
                bindings
                    .push_bind(&ac.commit_hash)
                    .push_bind(&ac.timestamp)
                    .push_bind(&ac.publication_version_id);
            });
            let query = query_builder.build();
            query.execute(&mut *self.tx).await?;
            query_builder.reset();
        }
        Ok(())
    }
}
