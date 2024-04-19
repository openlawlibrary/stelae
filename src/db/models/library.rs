use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Serialize)]
/// Model for library (collection).
pub struct Library {
    /// Materialized path to the library
    pub mpath: String,
}
