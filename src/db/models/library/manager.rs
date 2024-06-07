//! Manager for the library model.
use super::Library;
use crate::db::DatabaseTransaction;
use async_trait::async_trait;
use sqlx::QueryBuilder;

/// Size of the batch for bulk inserts.
const BATCH_SIZE: usize = 1000;

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Upsert a bulk of libraries into the database.
    ///
    /// # Errors
    /// Errors if the libraries cannot be inserted into the database.
    async fn insert_bulk(&mut self, libraries: Vec<Library>) -> anyhow::Result<()> {
        let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO library (mpath) ");
        for chunk in libraries.chunks(BATCH_SIZE) {
            query_builder.push_values(chunk, |mut bindings, lb| {
                bindings.push_bind(&lb.mpath);
            });
            let query = query_builder.build();
            query.execute(&mut *self.tx).await?;
        }
        Ok(())
    }
}
