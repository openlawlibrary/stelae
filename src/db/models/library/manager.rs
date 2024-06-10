//! Manager for the library model.
use super::Library;
use crate::db::{models::BATCH_SIZE, DatabaseConnection, DatabaseKind, DatabaseTransaction};
use async_trait::async_trait;
use sqlx::QueryBuilder;

#[async_trait]
impl super::Manager for DatabaseConnection {
    /// Find one library materialized path by url.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_lib_mpath_by_url(&self, url: &str) -> anyhow::Result<String> {
        let statement = "
            SELECT l.mpath
            FROM library l
            WHERE l.url = $1
            LIMIT 1
        ";
        let row = match self.kind {
            DatabaseKind::Postgres | DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, (String,)>(statement)
                    .bind(url)
                    .fetch_one(&mut *connection)
                    .await?
            }
        };
        Ok(row.0)
    }
}

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Upsert a bulk of libraries into the database.
    ///
    /// # Errors
    /// Errors if the libraries cannot be inserted into the database.
    async fn insert_bulk(&mut self, libraries: Vec<Library>) -> anyhow::Result<()> {
        let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO library ( mpath, url ) ");
        for chunk in libraries.chunks(BATCH_SIZE) {
            query_builder.push_values(chunk, |mut bindings, lb| {
                bindings.push_bind(&lb.mpath).push_bind(&lb.url);
            });
            let query = query_builder.build();
            query.execute(&mut *self.tx).await?;
        }
        Ok(())
    }
}
