//! Manager for the library change model.
use crate::db::{models::version::Version, DatabaseConnection, DatabaseKind, DatabaseTransaction};
use async_trait::async_trait;
use sqlx::QueryBuilder;

use super::LibraryChange;

/// Size of the batch for bulk inserts.
const BATCH_SIZE: usize = 1000;

#[async_trait]
impl super::Manager for DatabaseConnection {
    /// Find one library materialized path by url.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_lib_mpath_by_url(&self, url: &str) -> anyhow::Result<String> {
        let statement = "
            SELECT library_mpath
            FROM library_change
            WHERE url = $1
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
    /// All dates on which documents from this collection changed.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_all_collection_versions_by_mpath_and_publication(
        &self,
        mpath: &str,
        publication: &str,
    ) -> anyhow::Result<Vec<Version>> {
        let mut statement = "
            SELECT DISTINCT phpv.referenced_version as codified_date
            FROM changed_library_document cld
            LEFT JOIN publication_has_publication_versions phpv
                ON cld.publication = phpv.referenced_publication
                AND cld.version = phpv.referenced_version
            WHERE cld.library_mpath LIKE $1 AND phpv.publication = $2
        ";
        let mut rows = match self.kind {
            DatabaseKind::Postgres | DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, Version>(statement)
                    .bind(format!("{mpath}%"))
                    .bind(publication)
                    .fetch_all(&mut *connection)
                    .await?
            }
        };
        statement = "
            SELECT DISTINCT phpv.referenced_version as codified_date
            FROM library_change lc
            LEFT JOIN publication_has_publication_versions phpv
                ON lc.publication = phpv.referenced_publication
                AND lc.version = phpv.referenced_version
            WHERE lc.library_mpath LIKE $1 AND lc.status = 'Element added' AND phpv.publication = $2
            LIMIT 1
            ";
        let element_added = match self.kind {
            DatabaseKind::Postgres | DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, Version>(statement)
                    .bind(format!("{mpath}%"))
                    .bind(publication)
                    .fetch_one(&mut *connection)
                    .await
                    .ok()
            }
        };

        if let Some(el_added) = element_added {
            if !rows.contains(&el_added) {
                rows.push(el_added);
            }
        }
        rows.sort_by(|v1, v2| v2.codified_date.cmp(&v1.codified_date));
        Ok(rows)
    }
}

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Upsert a bulk of library changes into the database.
    ///
    /// # Errors
    /// Errors if the library changes cannot be inserted into the database.
    async fn insert_bulk(&mut self, library_changes: Vec<LibraryChange>) -> anyhow::Result<()> {
        let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO library_change (library_mpath, publication, version, stele, status, url) ");
        for chunk in library_changes.chunks(BATCH_SIZE) {
            query_builder.push_values(chunk, |mut bindings, lc| {
                bindings
                    .push_bind(&lc.library_mpath)
                    .push_bind(&lc.publication)
                    .push_bind(&lc.version)
                    .push_bind(&lc.stele)
                    .push_bind(&lc.status)
                    .push_bind(&lc.url);
            });
            let query = query_builder.build();
            query.execute(&mut *self.tx).await?;
        }
        Ok(())
    }
}
