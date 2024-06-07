//! Manager for the document model.
use crate::db::DatabaseTransaction;
use async_trait::async_trait;

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Upsert a new document into the database.
    ///
    /// # Errors
    /// Errors if the document cannot be inserted into the database.
    async fn create(&mut self, doc_id: &str) -> anyhow::Result<Option<i64>> {
        let statement = "
        INSERT OR IGNORE INTO document ( doc_id )
        VALUES ( $1 )
    ";
        let id = sqlx::query(statement)
            .bind(doc_id)
            .execute(&mut *self.tx)
            .await?
            .last_insert_id();
        Ok(id)
    }
}
