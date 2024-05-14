use serde::Deserialize;
/// Request for the versions endpoint.
#[derive(Deserialize, Debug)]
pub struct Version {
    /// Publication name.
    pub publication: Option<String>,
    /// Date to compare.
    pub date: Option<String>,
    /// Date to compare against.
    pub compare_date: Option<String>,
    /// Path to document/collection.
    pub path: Option<String>,
}
