//! Manager for the fonds model.
use crate::db::DatabaseTransaction;
use async_trait::async_trait;

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Upsert a new fonds into the database.
    ///
    /// # Errors
    /// Errors if the fonds cannot be inserted into the database.
    async fn create(&mut self, fonds: &str) -> anyhow::Result<Option<i64>> {
        let statement = "
            INSERT OR IGNORE INTO fonds ( name )
            VALUES ( $1 )
        ";
        let id = sqlx::query(statement)
            .bind(fonds)
            .execute(&mut *self.tx)
            .await?
            .last_insert_id();
        Ok(id)
    }

    /// Delete a fonds and all of its associated data via `ON DELETE CASCADE`.
    ///
    /// `PRAGMA foreign_keys = ON` is set at connection time, so a single
    /// delete on the `fonds` table cascades to all child tables automatically.
    ///
    /// # Errors
    /// Errors if the delete statement fails.
    async fn delete(&mut self, fonds: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM fonds WHERE name = $1")
            .bind(fonds)
            .execute(&mut *self.tx)
            .await?;
        Ok(())
    }
}
