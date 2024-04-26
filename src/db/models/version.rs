use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Trait for managing versions.
#[async_trait]
pub trait Manager {}

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug, Eq, PartialEq)]
/// Model for a version.
pub struct Version {
    /// Significant codified date of any publication.
    /// Used in the form %YYYY-%MM-%DD.
    pub codified_date: String,
}
