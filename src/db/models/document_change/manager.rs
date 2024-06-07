//! Manager for the document change model.
use super::DocumentChange;
use crate::db::{models::version::Version, DatabaseConnection, DatabaseKind, DatabaseTransaction};
use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::QueryBuilder;

/// Size of the batch for bulk inserts.
const BATCH_SIZE: usize = 1000;

#[async_trait]
impl super::Manager for DatabaseConnection {
    /// Find one document materialized path by url.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_doc_mpath_by_url(&self, url: &str) -> anyhow::Result<String> {
        let statement = "
            SELECT doc_mpath
            FROM document_change
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

    /// All dates on which given document changed.
    ///
    /// # Errors
    /// Errors if can't establish a connection to the database.
    async fn find_all_document_versions_by_mpath_and_publication(
        &self,
        mpath: &str,
        publication: &str,
    ) -> anyhow::Result<Vec<Version>> {
        let mut statement = "
            SELECT DISTINCT phpv.referenced_version as codified_date
            FROM document_change dc
            LEFT JOIN publication_has_publication_versions phpv
                ON dc.publication = phpv.referenced_publication
                AND dc.version = phpv.referenced_version
            WHERE dc.doc_mpath LIKE $1 AND phpv.publication = $2
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
            SELECT phpv.referenced_version as codified_date
            FROM document_change dc
            LEFT JOIN publication_has_publication_versions phpv
                ON dc.publication = phpv.referenced_publication
                AND dc.version = phpv.referenced_version
            WHERE dc.doc_mpath = $1
            AND phpv.publication = $2
            AND dc.status = 'Element added'
            LIMIT 1
        ";
        let element_added = match self.kind {
            DatabaseKind::Postgres | DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, Version>(statement)
                    .bind(mpath)
                    .bind(publication)
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
            SELECT phpv.referenced_version as codified_date
            FROM document_change dc
            LEFT JOIN publication_has_publication_versions phpv
                ON dc.publication = phpv.referenced_publication
                AND dc.version = phpv.referenced_version
            WHERE dc.doc_mpath = $1
            AND phpv.publication = $2
            AND dc.status = 'Element effective'
            LIMIT 1
        ";
        let mut doc = mpath.split('|').next().unwrap_or("").to_owned();
        doc.push('|');

        let document_effective = match self.kind {
            DatabaseKind::Postgres | DatabaseKind::Sqlite => {
                let mut connection = self.pool.acquire().await?;
                sqlx::query_as::<_, Version>(statement)
                    .bind(doc)
                    .bind(publication)
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
        let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO document_change (doc_mpath, status, url, change_reason, publication, version, stele, doc_id) ");
        for chunk in document_changes.chunks(BATCH_SIZE) {
            query_builder.push_values(chunk, |mut bindings, dc| {
                bindings
                    .push_bind(&dc.doc_mpath)
                    .push_bind(&dc.status)
                    .push_bind(&dc.url)
                    .push_bind(&dc.change_reason)
                    .push_bind(&dc.publication)
                    .push_bind(&dc.version)
                    .push_bind(&dc.stele)
                    .push_bind(&dc.doc_id);
            });
            let query = query_builder.build();
            query.execute(&mut *self.tx).await?;
            query_builder.reset();
        }
        Ok(())
    }
}
