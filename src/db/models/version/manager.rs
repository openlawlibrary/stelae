//! Manager for the version model.
use crate::db::DatabaseTransaction;
use async_trait::async_trait;

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Upsert a new version into the database.
    ///
    /// # Errors
    /// Errors if the version cannot be inserted into the database.
    async fn create(&mut self, codified_date: &str) -> anyhow::Result<Option<i64>> {
        let statement = "
            INSERT OR IGNORE INTO version ( codified_date )
            VALUES ( $1 )
        ";
        let id = sqlx::query(statement)
            .bind(codified_date)
            .execute(&mut *self.tx)
            .await?
            .last_insert_id();
        Ok(id)
    }
}
