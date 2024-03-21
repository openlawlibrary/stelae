use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Serialize)]
/// Model for a Stele.
pub struct PublicationVersion {
    /// Database id.
    pub id: i32,
    /// Date in a publication in %Y-%m-%d format
    pub version: String,
    /// Foreign key reference to publication id.
    pub publication_id: i32,
    /// Reason for building the publication.
    pub build_reason: Option<String>,
}
