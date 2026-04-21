//! Manager for the stele model.
use crate::db::DatabaseTransaction;
use async_trait::async_trait;

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Upsert a new stele into the database.
    ///
    /// # Errors
    /// Errors if the stele cannot be inserted into the database.
    async fn create(&mut self, stele: &str) -> anyhow::Result<Option<i64>> {
        let statement = "
            INSERT OR IGNORE INTO stele ( name )
            VALUES ( $1 )
        ";
        let id = sqlx::query(statement)
            .bind(stele)
            .execute(&mut *self.tx)
            .await?
            .last_insert_id();
        Ok(id)
    }

    /// Delete a stele and all of its associated data via `ON DELETE CASCADE`.
    ///
    /// `PRAGMA foreign_keys = ON` is set at connection time, so a single
    /// delete on the `stele` table cascades to all child tables automatically.
    ///
    /// # Errors
    /// Errors if the delete statement fails.
    async fn delete(&mut self, stele: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM stele WHERE name = $1")
            .bind(stele)
            .execute(&mut *self.tx)
            .await?;
        Ok(())
    }
}
