//! Manager for the document element model.
use async_trait::async_trait;
use sqlx::QueryBuilder;

use crate::db::{models::BATCH_SIZE, DatabaseConnection, DatabaseKind, DatabaseTransaction};

use super::DocumentElement;

#[async_trait]
impl super::Manager for DatabaseConnection {
    /// Find one document materialized path by url.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_doc_mpath_by_url(&self, url: &str, stele: &str) -> anyhow::Result<String> {
        let statement = "
            SELECT de.doc_mpath
            FROM document_element de
            WHERE de.url = $1 AND de.stele = $2
            LIMIT 1
        ";
        let row = match self.kind {
            DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, (String,)>(statement)
                    .bind(url)
                    .bind(stele)
                    .fetch_one(&mut *connection)
                    .await?
            }
        };
        Ok(row.0)
    }
}

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Upsert a bulk of document elements into the database.
    ///
    /// # Errors
    /// Errors if the document elements cannot be inserted into the database.
    async fn insert_bulk(&mut self, document_elements: Vec<DocumentElement>) -> anyhow::Result<()> {
        let mut query_builder = QueryBuilder::new(
            "INSERT OR IGNORE INTO document_element ( doc_mpath, url, doc_id, stele ) ",
        );
        for chunk in document_elements.chunks(BATCH_SIZE) {
            query_builder.push_values(chunk, |mut bindings, de| {
                bindings
                    .push_bind(&de.doc_mpath)
                    .push_bind(&de.url)
                    .push_bind(&de.doc_id)
                    .push_bind(&de.stele);
            });
            let query = query_builder.build();
            query.execute(&mut *self.tx).await?;
            query_builder.reset();
        }
        Ok(())
    }
}
