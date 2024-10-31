//! Manager for the library change model.
use crate::db::{
    models::{status::Status, version::Version, BATCH_SIZE},
    DatabaseConnection, DatabaseKind, DatabaseTransaction,
};
use async_trait::async_trait;
use sqlx::QueryBuilder;

use super::LibraryChange;

#[async_trait]
impl super::Manager for DatabaseConnection {
    /// All dates on which documents from this collection changed.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_all_collection_versions_by_mpath_and_publication(
        &self,
        mpath: &str,
        publication_id: &str,
    ) -> anyhow::Result<Vec<Version>> {
        let mut statement = "
            SELECT DISTINCT pv.version AS codified_date
            FROM changed_library_document cld
            LEFT JOIN document_change dc on cld.document_change_id = dc.id
            LEFT JOIN publication_has_publication_versions phpv ON dc.publication_version_id = phpv.publication_version_id
            LEFT JOIN publication_version pv ON phpv.publication_version_id = pv.id
            WHERE cld.library_mpath LIKE $1 AND phpv.publication_id = $2
        ";
        let mut rows = match self.kind {
            DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, Version>(statement)
                    .bind(format!("{mpath}%"))
                    .bind(publication_id)
                    .fetch_all(&mut *connection)
                    .await?
            }
        };
        statement = "
            SELECT DISTINCT pv.version AS codified_date
            FROM library_change lc
            LEFT JOIN publication_has_publication_versions phpv ON lc.publication_version_id = phpv.publication_version_id
            LEFT JOIN publication_version pv ON phpv.publication_version_id = pv.id
            WHERE lc.library_mpath LIKE $1 AND lc.status = $2 AND phpv.publication_id = $3
            LIMIT 1
        ";
        let element_added = match self.kind {
            DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, Version>(statement)
                    .bind(format!("{mpath}%"))
                    .bind(Status::ElementAdded.to_int())
                    .bind(publication_id)
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
        let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO library_change ( library_mpath, publication_version_id, status ) ");
        for chunk in library_changes.chunks(BATCH_SIZE) {
            query_builder.push_values(chunk, |mut bindings, lc| {
                bindings
                    .push_bind(&lc.library_mpath)
                    .push_bind(&lc.publication_version_id)
                    .push_bind(lc.status);
            });
            let query = query_builder.build();
            query.execute(&mut *self.tx).await?;
            query_builder.reset();
        }
        Ok(())
    }
}
