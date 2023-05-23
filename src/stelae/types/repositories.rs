use serde::{Deserialize, Deserializer};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repositories {
    /// Scopes of the repositories
    pub scopes: Option<Vec<String>>,
    /// Data repositories sorted by routes top down (most strict to least strict)
    #[serde(deserialize_with = "deserialize_repositories")]
    pub repositories: Vec<Repository>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repository {
    pub name: String,
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

/// Deserialize a map of repositories into a vector of sorted repositories
fn deserialize_repositories<'de, D>(deserializer: D) -> Result<Vec<Repository>, D::Error>
where
    D: Deserializer<'de>,
{
    let repositories: serde_json::Map<String, serde_json::Value> =
        serde_json::Map::deserialize(deserializer)?;
    let mut result = Vec::new();

    for (name, value) in repositories {
        let custom_value = value
            .get("custom")
            .ok_or_else(|| serde::de::Error::custom("Missing 'custom' field"))?;
        let custom: Custom = serde_json::from_value(custom_value.clone()).map_err(|e| {
            serde::de::Error::custom(format!("Failed to deserialize 'custom': {e}"))
        })?;
        result.push(Repository { name, custom });
    }
    // Sort the repositories by the length of their routes, longest first
    // This is needed because Actix routes are matched in the order they are added
    result.sort_by(|repo1, repo2| {
        let routes1 = repo1.custom.routes.as_ref().map_or(0, |r| {
            r.iter().map(std::string::String::len).max().unwrap_or(0)
        });
        let routes2 = repo2.custom.routes.as_ref().map_or(0, |r| {
            r.iter().map(std::string::String::len).max().unwrap_or(0)
        });
        routes2.cmp(&routes1)
    });
    Ok(result)
}
