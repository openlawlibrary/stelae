//! A Stele's data repositories.
use std::{collections::HashMap, fmt};

use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

/// Repositories object
///
/// Represents data repositories in a stele.
/// Repositories object is serialized from `repositories.json`.
///
/// `repositories.json` is expected to exist in /targets/repositories.json in the authentication repository.
/// Example `repositories.json`:
///
///
/// {
/// "scopes": ["us/ca/cities/san-mateo"],
/// "repositories": {
///    "`test_org_1/data_repo_1"`: {
///      "custom": {
///        "routes": ["example-route-glob-pattern-1"]
///      }
///    },
///    "`test_org_1/data_repo_2"`: {
///      "custom": {
///       "serve-prefix": "_prefix"
///      }
///  }
/// }
///
#[derive(Debug, Clone, Serialize, Default)]
pub struct Repositories {
    /// Scopes of the repositories
    pub scopes: Option<Vec<String>>,
    /// Map of repositories. The key is the name of the repository.
    pub repositories: HashMap<String, Repository>,
}

/// Repository object
///
/// Represents one concrete data repository in a stele.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Repository {
    /// Fully qualified name in `<org>/<name>` format.
    /// This is the key of the `repositories` entries.
    pub name: String,
    /// Custom object
    pub custom: Custom,
}

/// Custom object
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Custom {
    #[serde(rename = "type")]
    /// Type of data repository. e.g. `rdf`, `html`, `pdf`, `xml`, or any other.
    pub repository_type: Option<String>,
    /// "latest" or "historical". Currently not used by the framework.
    pub serve: String,
    /// Vector of glob patterns used by the Actix framework to resolve url routing.
    /// Routing to use when locating current blobs from the data repository.
    /// Example:
    ///
    /// Given a `["_underscore/.*"] glob pattern, the following urls are expected to be routed to the current data repository:
    ///
    /// - `/_underscore/`
    /// - `/_underscore/any/path`
    /// - `/_underscore/any/path/with/any/number/of/segments`
    pub routes: Option<Vec<String>>,
    #[serde(rename = "serve-prefix")]
    /// Prefix to use when serving the data repository.
    /// If `None`, the data repository will be served at the root.
    /// If `Some("prefix")`, the data repository will be served from `/prefix/<data>`.
    pub scope: Option<String>,
    /// Whether the data repository is a fallback.
    ///
    /// When a data repository is a fallback, it is used to serve current blobs when no other data repository matches the request.
    pub is_fallback: Option<bool>,
}

impl Repositories {
    /// Get the repositories sorted by the length of their routes, longest first.
    ///
    /// This is needed for serving current documents because Actix routes are matched in the order they are added.
    #[must_use]
    pub fn get_sorted_repositories(&self) -> Vec<&Repository> {
        let mut result = Vec::new();
        for repository in self.repositories.values() {
            result.push(repository);
        }
        result.sort_by(|repo1, repo2| {
            let routes1 = repo1.custom.routes.as_ref().map_or(0, |r| {
                r.iter().map(std::string::String::len).max().unwrap_or(0)
            });
            let routes2 = repo2.custom.routes.as_ref().map_or(0, |r| {
                r.iter().map(std::string::String::len).max().unwrap_or(0)
            });
            routes2.cmp(&routes1)
        });
        result
    }
}

#[allow(clippy::missing_trait_methods)]
impl<'de> Deserialize<'de> for Repositories {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// Visitor for the Repositories struct
        struct RepositoriesVisitor;

        impl<'de> Visitor<'de> for RepositoriesVisitor {
            type Value = Repositories;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Repositories")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Repositories, V::Error>
            where
                V: MapAccess<'de>,
            {
                Self::deserialize_repositories(&mut map)
            }
        }

        impl RepositoriesVisitor {
            /// Deserialize the repositories map from the `repositories.json` file.
            fn deserialize_repositories<'de, V>(map: &mut V) -> Result<Repositories, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut scopes = None;
                let mut repositories = HashMap::new();
                while let Some(key) = map.next_key()? {
                    match key {
                        "scopes" => {
                            scopes = map.next_value()?;
                        }
                        "repositories" => {
                            repositories = Self::deserialize_repositories_values(map)?;
                        }
                        _ => {
                            return Err(de::Error::unknown_field(key, &["scopes", "repositories"]))
                        }
                    }
                }
                Ok(Repositories {
                    scopes,
                    repositories,
                })
            }

            /// Deserialize individual repositories from the `repositories.json` file.
            fn deserialize_repositories_values<'de, V>(
                map: &mut V,
            ) -> Result<HashMap<String, Repository>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let repositories_json: HashMap<String, Value> = map.next_value()?;
                let mut repositories = HashMap::new();
                for (map_key, value) in repositories_json {
                    let custom_value = value
                        .get("custom")
                        .ok_or_else(|| serde::de::Error::custom("Missing 'custom' field"))?;
                    let custom: Custom =
                        serde_json::from_value(custom_value.clone()).map_err(|e| {
                            serde::de::Error::custom(format!("Failed to deserialize 'custom': {e}"))
                        })?;
                    let repo = Repository {
                        name: map_key.clone(),
                        custom,
                    };
                    repositories.insert(map_key, repo);
                }
                Ok(repositories)
            }
        }
        /// Expected fields in the `repositories.json` file.
        const FIELDS: &[&str] = &["scopes", "repositories"];
        deserializer.deserialize_struct("Repositories", FIELDS, RepositoriesVisitor)
    }
}
