use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Serialize, Clone)]
/// Model for publication which contain publication versions.
pub struct PublicationHasPublicationVersions {
    /// Foreign key reference to publication name.
    pub publication: String,
    /// Publication can reference another publication.
    /// Foreign key reference to the referenced publication name.
    pub referenced_publication: String,
    /// Date in a publication in %Y-%m-%d format
    pub version: String,
    /// Foreign key reference to stele.
    pub stele: String,
}
