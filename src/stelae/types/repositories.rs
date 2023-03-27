use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repositories {
    scopes: Vec<String>,
    repositories: HashMap<String, Repository>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Repository {
    custom: Custom,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Custom {
    #[serde(rename = "type")]
    repository_type: Option<String>,
    #[serde(rename = "allow-unauthenticated-commits")]
    allow_unauthenticated_commits: bool,
    serve: String,
    routes: Vec<String>,
    #[serde(rename = "serve-prefix")]
    scope: Option<String>,
}
