use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Serialize, Hash, Eq, PartialEq, Clone)]
/// Model for a Stele.
pub struct PublicationVersion {
    /// Date in a publication in %Y-%m-%d format
    pub version: String,
    /// Foreign key reference to publication name.
    pub name: String,
    /// Foreign key reference to stele.
    pub stele: String,
    /// Reason for building the publication.
    pub build_reason: Option<String>,
}
