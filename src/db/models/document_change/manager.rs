//! Manager for the document change model.
use super::DocumentChange;
use crate::db::{
    models::{status::Status, version::Version, BATCH_SIZE},
    DatabaseConnection, DatabaseKind, DatabaseTransaction,
};
use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::QueryBuilder;

#[async_trait]
impl super::Manager for DatabaseConnection {
    /// All dates on which given document changed.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_all_document_versions_by_mpath_and_publication(
        &self,
        mpath: &str,
        publication_id: &str,
    ) -> anyhow::Result<Vec<Version>> {
        let mut statement = "
            SELECT DISTINCT pv.version AS codified_date
            FROM document_change dc
            LEFT JOIN publication_has_publication_versions phpv ON dc.publication_version_id = phpv.publication_version_id
            LEFT JOIN publication_version pv ON phpv.publication_version_id = pv.id
            WHERE dc.doc_mpath LIKE $1 AND phpv.publication_id = $2
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
            SELECT pv.version AS codified_date
            FROM document_change dc
            LEFT JOIN publication_has_publication_versions phpv ON dc.publication_version_id = phpv.publication_version_id
            LEFT JOIN publication_version pv ON phpv.publication_version_id = pv.id
            WHERE dc.doc_mpath = $1 AND phpv.publication_id = $2 AND dc.status = $3
            LIMIT 1
        ";
        let element_added = match self.kind {
            DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, Version>(statement)
                    .bind(mpath)
                    .bind(publication_id)
                    .bind(Status::ElementAdded.to_int())
                    .fetch_one(&mut *connection)
                    .await
                    .ok()
            }
        };

        if element_added.is_none() {
            // When element doesn't have date added, it means we're looking
            // at an old publication and this element doesn't yet exist in it
            rows.sort_by(|v1, v2| v2.codified_date.cmp(&v1.codified_date));
            return Ok(rows);
        }

        statement = "
            SELECT pv.version AS codified_date
            FROM document_change dc
            LEFT JOIN publication_has_publication_versions phpv ON dc.publication_version_id = phpv.publication_version_id
            LEFT JOIN publication_version pv ON phpv.publication_version_id = pv.id
            WHERE dc.doc_mpath = $1 AND phpv.publication_id = $2 AND dc.status = $3
            LIMIT 1
        ";
        let mut doc = mpath.split('|').next().unwrap_or("").to_owned();
        doc.push('|');

        let document_effective = match self.kind {
            DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, Version>(statement)
                    .bind(doc)
                    .bind(publication_id)
                    .bind(Status::ElementEffective.to_int())
                    .fetch_one(&mut *connection)
                    .await
                    .ok()
            }
        };

        if let (Some(doc_effective), Some(el_added)) = (document_effective, element_added) {
            if !rows.contains(&doc_effective)
                && NaiveDate::parse_from_str(&doc_effective.codified_date, "%Y-%m-%d")
                    .unwrap_or_default()
                    > NaiveDate::parse_from_str(&el_added.codified_date, "%Y-%m-%d")
                        .unwrap_or_default()
            {
                rows.push(doc_effective);
            }
        }
        rows.sort_by(|v1, v2| v2.codified_date.cmp(&v1.codified_date));
        Ok(rows)
    }
}

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Upsert a bulk of document changes into the database.
    ///
    /// # Errors
    /// Errors if the document changes cannot be inserted into the database.
    async fn insert_bulk(&mut self, document_changes: Vec<DocumentChange>) -> anyhow::Result<()> {
        let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO document_change ( id, status, change_reason, publication_version_id, doc_mpath ) ");
        for chunk in document_changes.chunks(BATCH_SIZE) {
            query_builder.push_values(chunk, |mut bindings, dc| {
                bindings
                    .push_bind(&dc.id)
                    .push_bind(dc.status)
                    .push_bind(&dc.change_reason)
                    .push_bind(&dc.publication_version_id)
                    .push_bind(&dc.doc_mpath);
            });
            let query = query_builder.build();
            query.execute(&mut *self.tx).await?;
            query_builder.reset();
        }
        Ok(())
    }
}
