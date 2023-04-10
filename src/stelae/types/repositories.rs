use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repositories {
    /// Scopes of the repositories
    pub scopes: Option<Vec<String>>,
    /// Data repositories sorted by routes top down (most strict to least strict)
    pub repositories: BTreeMap<String, Repository>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repository {
    pub custom: Custom,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Custom {
    #[serde(rename = "type")]
    pub repository_type: Option<String>,
    #[serde(rename = "allow-unauthenticated-commits")]
    pub allow_unauthenticated_commits: Option<bool>,
    pub serve: String,
    pub routes: Option<Vec<String>>,
    #[serde(rename = "serve-prefix")]
    pub scope: Option<String>,
    pub is_fallback: Option<bool>,
}
