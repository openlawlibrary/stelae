//! Manager for the changed library document model.
use super::ChangedLibraryDocument;
use crate::db::DatabaseTransaction;
use async_trait::async_trait;
use sqlx::QueryBuilder;

/// Size of the batch for bulk inserts.
const BATCH_SIZE: usize = 1000;

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Upsert a bulk of changed library documents into the database.
    ///
    /// # Errors
    /// Errors if the changed library documents cannot be inserted into the database.
    async fn insert_bulk(
        &mut self,
        changed_library_document: Vec<ChangedLibraryDocument>,
    ) -> anyhow::Result<()> {
        let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO changed_library_document (publication, version, stele, doc_mpath, status, library_mpath, url) ");
        for chunk in changed_library_document.chunks(BATCH_SIZE) {
            query_builder.push_values(chunk, |mut bindings, cl| {
                bindings
                    .push_bind(&cl.publication)
                    .push_bind(&cl.version)
                    .push_bind(&cl.stele)
                    .push_bind(&cl.doc_mpath)
                    .push_bind(&cl.status)
                    .push_bind(&cl.library_mpath)
                    .push_bind(&cl.url);
            });
            let query = query_builder.build();
            query.execute(&mut *self.tx).await?;
        }
        Ok(())
    }
}
