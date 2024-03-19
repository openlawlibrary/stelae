use serde::{Deserialize, Serialize};
use sqlx::types::chrono;

#[derive(sqlx::FromRow, Deserialize, Serialize)]
/// Model for a Stele.
pub struct Publication {
    /// Database id.
    pub id: i32,
    /// Name of the publication in %YYYY-%MM-%DD format
    /// with optionally incrementing version numbers
    /// when two publications exist on same date.
    pub name: String,
    /// Date of the publication.
    pub date: chrono::NaiveDate,
    /// FK reference to Stele id.
    pub stele_id: i32,
    /// Whether the publication has been revoked.
    /// A publication is revoked if another publication exists
    /// on the same date with a higher version number.
    pub revoked: bool,
}
