//! Manager for the publication model.
use crate::db::{DatabaseConnection, DatabaseKind, DatabaseTransaction};
use async_trait::async_trait;
use chrono::NaiveDate;

use super::Publication;

#[async_trait]
impl super::Manager for DatabaseConnection {
    /// Find all publications which are not revoked for a given stele.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_all_non_revoked_publications(
        &self,
        stele: &str,
    ) -> anyhow::Result<Vec<Publication>> {
        let statement = "
            SELECT *
            FROM publication
            WHERE revoked = 0 AND stele = $1
            ORDER BY name DESC
        ";
        let rows = match self.kind {
            DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, Publication>(statement)
                    .bind(stele)
                    .fetch_all(&mut *connection)
                    .await?
            }
        };
        Ok(rows)
    }
}

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Upsert a new publication into the database.
    ///
    /// # Errors
    /// Errors if the publication cannot be inserted into the database.
    async fn create(
        &mut self,
        hash_id: &str,
        name: &str,
        date: &NaiveDate,
        stele: &str,
        last_valid_publication_id: Option<String>,
        last_valid_version: Option<String>,
    ) -> anyhow::Result<Option<i64>> {
        let statement = "
            INSERT OR IGNORE INTO publication ( id, name, date, stele, revoked, last_valid_publication_id, last_valid_version )
            VALUES ( $1, $2, $3, $4, FALSE, $5, $6)
        ";
        let id = sqlx::query(statement)
            .bind(hash_id)
            .bind(name)
            .bind(date.to_string())
            .bind(stele)
            .bind(last_valid_publication_id)
            .bind(last_valid_version)
            .execute(&mut *self.tx)
            .await?
            .last_insert_id();
        Ok(id)
    }
    /// Update a publication by name and stele to be revoked.
    ///
    /// # Errors
    /// Errors if the publication cannot be updated.
    async fn update_by_name_and_stele_set_revoked_true(
        &mut self,
        name: &str,
        stele: &str,
    ) -> anyhow::Result<()> {
        let statement = "
            UPDATE publication
            SET revoked = TRUE
            WHERE name = $1 AND stele = $2
        ";
        sqlx::query(statement)
            .bind(name)
            .bind(stele)
            .execute(&mut *self.tx)
            .await?;
        Ok(())
    }
    /// Find the last non-revoked publication by `stele_id`.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_last_inserted(&mut self, stele: &str) -> anyhow::Result<Option<Publication>> {
        let statement = "
            SELECT *
            FROM publication
            WHERE revoked = 0 AND stele = $1
            ORDER BY date DESC
            LIMIT 1
        ";
        let row = sqlx::query_as::<_, Publication>(statement)
            .bind(stele)
            .fetch_one(&mut *self.tx)
            .await
            .ok();
        Ok(row)
    }

    /// Find a publication by `name` and `stele_id`.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_by_name_and_stele(
        &mut self,
        name: &str,
        stele: &str,
    ) -> anyhow::Result<Option<Publication>> {
        let statement = "
            SELECT *
            FROM publication
            WHERE name = $1 AND stele = $2 AND revoked = 0
        ";
        let row = sqlx::query_as::<_, Publication>(statement)
            .bind(name)
            .bind(stele)
            .fetch_one(&mut *self.tx)
            .await
            .ok();
        Ok(row)
    }

    /// Filter publications by `name` and `stele_id` which is not revoked.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_first_by_name_and_stele_non_revoked(
        &mut self,
        name: &str,
        stele: &str,
    ) -> anyhow::Result<Publication> {
        let statement = "
            SELECT *
            FROM publication
            WHERE name = $1 AND stele = $2 AND revoked = 0
        ";
        let row = sqlx::query_as::<_, Publication>(statement)
            .bind(name)
            .bind(stele)
            .fetch_one(&mut *self.tx)
            .await?;
        Ok(row)
    }

    /// Find all publication names by date and stele.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_all_by_date_and_stele_order_by_name_desc(
        &mut self,
        date: String,
        stele: String,
    ) -> anyhow::Result<Vec<Publication>> {
        let statement = "
            SELECT *
            FROM publication
            WHERE date = $1 AND stele = $2
            ORDER BY name DESC
        ";
        let rows = sqlx::query_as::<_, Publication>(statement)
            .bind(date)
            .bind(stele)
            .fetch_all(&mut *self.tx)
            .await?;
        Ok(rows)
    }
}
