use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod manager;

/// Trait for managing transactions on publication has publication versions.
#[async_trait]
pub trait TxManager {
    /// Upsert a bulk of `publication_has_publication_versions` into the database.
    async fn insert_bulk(
        &mut self,
        publication_has_publication_versions: Vec<PublicationHasPublicationVersions>,
    ) -> anyhow::Result<()>;
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Clone, Debug)]
/// Model for publication which contain publication versions.
pub struct PublicationHasPublicationVersions {
    /// Foreign key reference to publication id.
    pub publication_id: String,
    /// Publication can reference another publication.
    /// Foreign key reference to the referenced publication version id.
    pub publication_version_id: String,
}
