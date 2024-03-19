use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Serialize)]
/// Model for a Stele.
pub struct Stele {
    /// Database id.
    pub id: i32,
    /// Stele identifier in <org>/<name> format.
    /// Example: `org-name/repo-name-law`.
    pub name: String,
}
