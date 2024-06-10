//! Manager for the changed library document model.
use super::ChangedLibraryDocument;
use crate::db::{models::BATCH_SIZE, DatabaseTransaction};
use async_trait::async_trait;
use sqlx::QueryBuilder;

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
        let mut query_builder = QueryBuilder::new(
            "INSERT OR IGNORE INTO changed_library_document ( library_mpath, document_change_id ) ",
        );
        for chunk in changed_library_document.chunks(BATCH_SIZE) {
            query_builder.push_values(chunk, |mut bindings, cl| {
                bindings
                    .push_bind(&cl.library_mpath)
                    .push_bind(&cl.document_change_id);
            });
            let query = query_builder.build();
            query.execute(&mut *self.tx).await?;
        }
        Ok(())
    }
}
