use serde::{Deserialize, Serialize};
use sqlx::types::chrono;

#[derive(sqlx::FromRow, Deserialize, Serialize)]
/// Model for a Stele.
pub struct Publication {
    /// Name of the publication in %YYYY-%MM-%DD format
    /// with optionally incrementing version numbers
    /// when two publications exist on same date.
    pub name: String,
    /// Date of the publication.
    pub date: chrono::NaiveDate,
    /// Foreign key reference to stele by name.
    pub stele: String,
    /// Whether the publication has been revoked.
    /// A publication is revoked if another publication exists
    /// on the same date with a higher version number.
    pub revoked: bool,
    /// If a publication is derived from another publication,
    /// represents the last publication name that was valid before this publication.
    pub last_valid_publication_name: Option<String>,
    /// If a publication is derived from another publication,
    /// represents the last publication version (codified date) from the previous publication
    /// that the current publication is derived from.
    pub last_valid_version: Option<String>
}
