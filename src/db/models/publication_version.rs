use serde::{Deserialize, Serialize};
use sqlx::{any::AnyRow, FromRow, Row};

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
