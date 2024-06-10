//! Manager for the `publication_has_publication_versions` table.
use crate::db::{models::BATCH_SIZE, DatabaseTransaction};
use async_trait::async_trait;
use sqlx::QueryBuilder;

use super::PublicationHasPublicationVersions;

#[async_trait]
impl super::TxManager for DatabaseTransaction {
    /// Upsert a bulk of `publication_has_publication_versions` into the database.
    ///
    /// # Errors
    /// Errors if the `publication_has_publication_versions` cannot be inserted into the database.
    async fn insert_bulk(
        &mut self,
        publication_has_publication_versions: Vec<PublicationHasPublicationVersions>,
    ) -> anyhow::Result<()> {
        let mut query_builder = QueryBuilder::new("INSERT OR IGNORE INTO publication_has_publication_versions ( publication_id, publication_version_id ) ");
        for chunk in publication_has_publication_versions.chunks(BATCH_SIZE) {
            query_builder.push_values(chunk, |mut bindings, pb| {
                bindings
                    .push_bind(&pb.publication_id)
                    .push_bind(&pb.publication_version_id);
            });
            let query = query_builder.build();
            query.execute(&mut *self.tx).await?;
        }
        Ok(())
    }
}
