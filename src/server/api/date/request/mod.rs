use serde::Deserialize;
/// Request for the date endpoint.
#[derive(Deserialize, Debug)]
pub struct Date {
    /// Version date.
    pub version_date: Option<String>,
    /// Path to document/collection.
    pub path: Option<String>,
}
