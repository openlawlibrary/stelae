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
        publication: &str,
        codified_date: &str,
        stele: &str,
    ) -> anyhow::Result<Option<i64>>;
    /// Find the last inserted publication version by a `stele` and `publication`.
    async fn find_last_inserted_by_publication_and_stele(
        &mut self,
        publication: &str,
        stele: &str,
    ) -> anyhow::Result<Option<PublicationVersion>>;
    /// Find all publication versions by a `publication` and `stele`.
    async fn find_all_by_publication_name_and_stele(
        &mut self,
        publication: &str,
        stele: &str,
    ) -> anyhow::Result<Vec<PublicationVersion>>;
    /// Find all publication has publication versions by a `publication` and `stele` recursively.
    async fn find_all_in_publication_has_publication_versions(
        &mut self,
        publications: Vec<String>,
        stele: &str,
    ) -> anyhow::Result<Vec<PublicationVersion>>;
    /// Find all publication versions by a `publication` and `stele` recursively.
    async fn find_all_recursive_for_publication(
        &mut self,
        publication_name: String,
        stele: String,
    ) -> anyhow::Result<Vec<PublicationVersion>>;
}

#[derive(Deserialize, Serialize, Hash, Eq, PartialEq, Clone)]
/// Model for a Stele.
pub struct PublicationVersion {
    /// Date in a publication in %Y-%m-%d format
    pub version: String,
    /// Foreign key reference to publication name.
    pub publication: String,
    /// Foreign key reference to stele.
    pub stele: String,
    /// Reason for building the publication.
    pub build_reason: Option<String>,
}

impl FromRow<'_, AnyRow> for PublicationVersion {
    fn from_row(row: &AnyRow) -> anyhow::Result<Self, sqlx::Error> {
        Ok(Self {
            version: row.try_get("version")?,
            publication: row.try_get("publication")?,
            stele: row.try_get("stele")?,
            build_reason: row.try_get("build_reason").ok(),
        })
    }
}
