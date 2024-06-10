use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{any::AnyRow, FromRow, Row};

pub mod manager;

/// Trait for managing transactions on publication versions.
#[async_trait]
pub trait TxManager {
    /// Create a new publication version.
    async fn create(
        &mut self,
        hash_id: &str,
        publication_id: &str,
        codified_date: &str,
    ) -> anyhow::Result<Option<i64>>;
    /// Find the last inserted publication version by a `stele` and `publication`.
    async fn find_last_inserted_date_by_publication_id(
        &mut self,
        publication_id: &str,
    ) -> anyhow::Result<Option<PublicationVersion>>;
    /// Find all publication versions by a `publication_id`.
    async fn find_all_by_publication_id(
        &mut self,
        publication_id: &str,
    ) -> anyhow::Result<Vec<PublicationVersion>>;
    /// Find all publication has publication versions by `publication` ids.
    async fn find_all_in_publication_has_publication_versions(
        &mut self,
        publication_ids: Vec<String>,
    ) -> anyhow::Result<Vec<PublicationVersion>>;
    /// Find all publication versions by a `publication` and `stele` recursively.
    async fn find_all_recursive_for_publication(
        &mut self,
        publication_id: String,
    ) -> anyhow::Result<Vec<PublicationVersion>>;
}

#[derive(Deserialize, Serialize, Hash, Eq, PartialEq, Clone)]
/// Model for a Stele.
pub struct PublicationVersion {
    /// A hashed identifier for the publication version.
    /// The hash is generated from the `publication` name, `version` date  and `stele` fields.
    pub id: String,
    /// Date in a publication in %Y-%m-%d format
    pub version: String,
    /// Foreign key reference to the publication table.
    pub publication_id: String,
    /// Reason for building the publication.
    pub build_reason: Option<String>,
}

impl FromRow<'_, AnyRow> for PublicationVersion {
    fn from_row(row: &AnyRow) -> anyhow::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            version: row.try_get("version")?,
            publication_id: row.try_get("publication_id")?,
            build_reason: row.try_get("build_reason").ok(),
        })
    }
}
