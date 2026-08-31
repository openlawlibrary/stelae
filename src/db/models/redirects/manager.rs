//! Manager for the redirects model.
use crate::db::{
    models::{redirects::RedirectPair, BATCH_SIZE},
    DatabaseConnection, DatabaseKind, DatabaseTransaction,
};
use async_trait::async_trait;
use sqlx::QueryBuilder;

#[async_trait]
impl super::Manager for DatabaseConnection {
    /// Finds a redirect target for a given URL.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database or no query does not return a row.
    async fn find_redirect_for_url(
        &self,
        stele: String,
        repo_name: String,
        from_url: String,
    ) -> anyhow::Result<String> {
        let statement = "
            SELECT from_url, to_url
            FROM redirects
            WHERE stele_name = $1
              AND repo_name = $2
              AND from_url = $3
        ";
        let row = match self.kind {
            DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, RedirectPair>(statement)
                    .bind(stele)
                    .bind(repo_name)
                    .bind(from_url)
                    .fetch_one(&mut *connection)
                    .await?
            }
        };

        Ok(row.to_url)
    }
}

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Upsert a bulk of redirects into the database.
    ///
    /// # Errors
    /// Errors if the redirects cannot be inserted into the database.
    async fn insert_bulk(
        &mut self,
        stele: &str,
        repo_name: &str,
        redirect_pairs: Vec<RedirectPair>,
    ) -> anyhow::Result<()> {
        let mut query_builder = QueryBuilder::new(
            "INSERT OR IGNORE INTO redirects ( stele_name, repo_name, from_url, to_url ) ",
        );
        for chunk in redirect_pairs.chunks(BATCH_SIZE) {
            query_builder.push_values(chunk, |mut bindings, rp| {
                bindings
                    .push_bind(stele)
                    .push_bind(repo_name)
                    .push_bind(&rp.from_url)
                    .push_bind(&rp.to_url);
            });
            let query = query_builder.build();
            query.execute(&mut *self.tx).await?;
            query_builder.reset();
        }
        Ok(())
    }
}
