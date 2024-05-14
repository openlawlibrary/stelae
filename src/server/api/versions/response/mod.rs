use std::{cmp::Reverse, collections::BTreeMap};

use serde::Deserialize;
use serde::Serialize;

use crate::db::models;

use self::messages::Historical;

use super::format_date;
use super::CURRENT_PUBLICATION_NAME;

/// Historical messages for the versions endpoint.
pub mod messages;

/// Response for the versions endpoint.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Versions {
    /// Currently selected publication.
    /// Resolves to "Current" if the latest publication is selected.
    pub active_publication: String,
    /// Currently selected version.
    /// Resolves to "current" if the latest version is selected.
    pub active_version: String,
    /// Currently selected version to compare against.
    /// If compare_date is specified, this will be the date to compare against.
    pub active_compare_to: Option<String>,
    /// Features for the versions endpoint.
    pub features: Features,
    /// URL path.
    pub path: String,
    /// List of all found publications in descending order.
    pub publications: BTreeMap<Reverse<String>, Publication>,
    /// Messages for the versions endpoint.
    pub messages: Historical,
}

/// Features for the versions endpoint.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Features {
    /// Whether the compare feature is enabled.
    pub compare: bool,
    /// Whether the historical versions feature is enabled.
    pub historical_versions: bool,
}

/// Response for a publication.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Publication {
    /// Whether the publication is currently active.
    pub active: bool,
    /// Date of the publication.
    pub date: String,
    /// Display name of the publication.
    pub display: String,
    /// Name of the publication.
    pub name: String,
    /// List of versions for the publication.
    pub versions: Vec<Version>,
}

/// Response for a version.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    /// Codified date of the version.
    pub date: String,
    /// Display date of the version.
    pub display: String,
    /// Version number of the version.
    #[serde(rename = "version")]
    pub index: usize,
}

impl From<models::version::Version> for Version {
    fn from(value: models::version::Version) -> Self {
        Self {
            date: value.codified_date.clone(),
            display: value.codified_date,
            index: 0,
        }
    }
}

impl Versions {
    /// Build and returns an HTTP versions response converted into json.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn build(
        active_publication_name: &str,
        active_version: String,
        active_compare_to: Option<String>,
        url: &str,
        publications: &[models::publication::Publication],
        current_publication_name: &str,
        versions: &[Version],
        messages: Historical,
    ) -> Self {
        Self {
            active_publication: active_publication_name.to_owned(),
            active_version,
            active_compare_to,
            features: Features {
                compare: true,
                historical_versions: true,
            },
            path: url.strip_prefix('/').unwrap_or_default().to_owned(),
            publications: {
                let mut sorted_publications = BTreeMap::new();
                for pb in publications {
                    sorted_publications.insert(
                        Reverse(pb.name.clone()),
                        Publication {
                            active: pb.name == active_publication_name,
                            date: pb.date.clone(),
                            display: Self::format_display_date(
                                &pb.name,
                                &pb.date,
                                current_publication_name,
                            ),
                            name: pb.name.clone(),
                            versions: {
                                if pb.name == active_publication_name {
                                    versions.to_vec()
                                } else {
                                    vec![]
                                }
                            },
                        },
                    );
                }
                sorted_publications
            },
            messages,
        }
    }

    /// Returns a formatted display date.
    /// If the `date` is current, returns the date with `(current)` appended.
    fn format_display_date(name: &str, date: &str, current_date: &str) -> String {
        if name == CURRENT_PUBLICATION_NAME {
            CURRENT_PUBLICATION_NAME.to_owned()
        } else {
            let mut formatted_date = format_date(date);
            if date == current_date {
                formatted_date.push_str(" (current)");
            }
            formatted_date
        }
    }
}

impl Version {
    /// Create a new version.
    #[must_use]
    pub const fn new(date: String, display: String, index: usize) -> Self {
        Self {
            date,
            display,
            index,
        }
    }

    /// Insert a new version if it is not present in the list of versions.
    /// If the date is not in the list of versions, add it
    /// This for compatibility purposes with the previous implementation of historical versions
    pub fn insert_if_not_present(versions: &mut Vec<Self>, date: Option<String>) {
        if let Some(version_date) = date {
            if versions.iter().all(|ver| ver.date != version_date) {
                let version = Self::new(version_date.clone(), version_date, 0);
                Self::insert_version_sorted(versions, version);
            }
        }
    }

    /// Insert a new item into an already sorted collection.
    /// The collection is sorted by date in descending order.
    pub fn insert_version_sorted(collection: &mut Vec<Self>, item: Self) {
        let mut idx = 0;
        for i in collection.iter() {
            if i.date < item.date {
                break;
            }
            idx += 1;
        }
        collection.insert(idx, item);
    }

    /// Utility function to find the index of a date in a list of versions.
    #[must_use]
    pub fn find_index_or_closest(versions: &[Self], date: &str) -> usize {
        versions
            .iter()
            .position(|ver| ver.date.as_str() == date)
            .unwrap_or_else(|| {
                let closest_date = versions
                    .iter()
                    .filter(|ver| ver.date.as_str() < date)
                    .max_by(|current, next| current.date.cmp(&next.date))
                    .map_or_else(|| None, |ver| Some(ver.date.as_str()))
                    .unwrap_or("-1");
                versions
                    .iter()
                    .position(|ver| ver.date.as_str() == closest_date)
                    .unwrap_or(versions.len())
            })
    }
}
