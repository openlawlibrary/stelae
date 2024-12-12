//! Targets metadata types.
//! Represents the metadata in the `targets/<org>/<data-repo>` file.
use serde::Deserialize;
use serde_derive::Serialize;

/// An entry in the `targets/<org>/<data-repo>` file.
///
/// Example:
/// ```rust
/// use stelae::stelae::types::targets_metadata::TargetsMetadata;
/// use serde_json::json;
///
/// let data = r#"
/// {
///    "branch": "publication/2024-09-16",
///    "build-date": "2024-09-16",
///    "commit": "1b7334f58f41a53d6e4d9fc11fba6793cb22eb36",
///    "codified-date": "2024-10-03"
/// }
/// "#;
/// let targets_metadata: TargetsMetadata = serde_json::from_str(data).unwrap();
/// assert_eq!(targets_metadata.branch, "publication/2024-09-16");
/// assert_eq!(targets_metadata.build_date.unwrap(), "2024-09-16");
/// assert_eq!(targets_metadata.commit, "1b7334f58f41a53d6e4d9fc11fba6793cb22eb36");
/// assert_eq!(targets_metadata.codified_date.unwrap(), "2024-10-03");
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct TargetsMetadata {
    /// A git branch name.
    pub branch: String,
    /// The date the build was created.
    #[serde(rename = "build-date")]
    pub build_date: Option<String>,
    /// The commit hash.
    pub commit: String,
    /// The date the code was codified.
    #[serde(rename = "codified-date")]
    pub codified_date: Option<String>,
}
