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
    #[allow(clippy::unwrap_in_result, clippy::unwrap_used)]
    fn from_row(row: &AnyRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            version: row.try_get("version").unwrap(),
            publication: row.try_get("publication").unwrap(),
            stele: row.try_get("stele").unwrap(),
            build_reason: row.try_get("build_reason").ok(),
        })
    }
}
