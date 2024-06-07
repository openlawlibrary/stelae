use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::{any::AnyRow, FromRow, Row};

pub mod manager;

/// Trait for managing publications.
#[async_trait]
pub trait Manager {
    /// Find all publications which are not revoked for a given stele.
    async fn find_all_non_revoked_publications(
        &self,
        stele: &str,
    ) -> anyhow::Result<Vec<Publication>>;
}

/// Trait for managing transactions on publications.
#[async_trait]
pub trait TxManager {
    /// Create a new publication.
    async fn create(
        &mut self,
        name: &str,
        date: &NaiveDate,
        stele: &str,
        last_valid_publication_name: Option<String>,
        last_valid_version: Option<String>,
    ) -> anyhow::Result<Option<i64>>;
    /// Update a publication by name and set revoked to true.
    async fn update_by_name_and_stele_set_revoked_true(
        &mut self,
        name: &str,
        stele: &str,
    ) -> anyhow::Result<()>;
    /// Find the last inserted publication for a given stele.
    async fn find_last_inserted(&mut self, stele: &str) -> anyhow::Result<Option<Publication>>;
    /// Find a publication by name and stele.
    async fn find_by_name_and_stele(
        &mut self,
        name: &str,
        stele: &str,
    ) -> anyhow::Result<Publication>;
    /// Find all by date and stele and sort by name in descending order.
    /// Used in revocation logic to find the latest publication.
    async fn find_all_by_date_and_stele_order_by_name_desc(
        &mut self,
        date: String,
        stele: String,
    ) -> anyhow::Result<Vec<Publication>>;
}

#[derive(Deserialize, Serialize, Debug)]
/// Model for a Stele.
pub struct Publication {
    /// Name of the publication in %YYYY-%MM-%DD format
    /// with optionally incrementing version numbers
    /// when two publications exist on same date.
    pub name: String,
    /// Date of the publication.
    pub date: String,
    /// Foreign key reference to stele by name.
    pub stele: String,
    /// Whether the publication has been revoked.
    /// A publication is revoked if another publication exists
    /// on the same date with a higher version number.
    pub revoked: i64,
    /// If a publication is derived from another publication,
    /// represents the last publication name that was valid before this publication.
    pub last_valid_publication_name: Option<String>,
    /// If a publication is derived from another publication,
    /// represents the last publication version (codified date) from the previous publication
    /// that the current publication is derived from.
    pub last_valid_version: Option<String>,
}

impl FromRow<'_, AnyRow> for Publication {
    fn from_row(row: &AnyRow) -> anyhow::Result<Self, sqlx::Error> {
        Ok(Self {
            name: row.try_get("name")?,
            date: row.try_get("date")?,
            stele: row.try_get("stele")?,
            revoked: row.try_get("revoked")?,
            last_valid_publication_name: row.try_get("last_valid_publication_name").ok(),
            last_valid_version: row.try_get("last_valid_version").ok(),
        })
    }
}

impl Publication {
    /// Create a new publication.
    #[must_use]
    pub const fn new(name: String, date: String, stele: String) -> Self {
        Self {
            name,
            date,
            stele,
            revoked: 0,
            last_valid_publication_name: None,
            last_valid_version: None,
        }
    }
}
