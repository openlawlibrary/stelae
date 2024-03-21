use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Serialize)]
/// Model for documents.
pub struct Document {
    /// Unique document identifier.
    pub doc_id: String
}
