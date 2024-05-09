//! Handlers for serving historical documents.
#![allow(clippy::future_not_send)]
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::NaiveDate;
use std::convert::Into;

use crate::{
    db::{
        models::{
            document_change, library_change,
            publication::{self, Publication},
        },
        DatabaseConnection,
    },
    server::app::{AppState, GlobalState},
    stelae::archive::Archive,
};

/// Name of the current publication.
pub const CURRENT_PUBLICATION_NAME: &str = "Current";
/// Name of the current version.
pub const CURRENT_VERSION_NAME: &str = "Current";
/// Date of the current version.
pub const CURRENT_VERSION_DATE: &str = "current";

/// Module that maps the HTTP web request body to structs.
mod request {
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
}

/// Module that maps the HTTP web response to structs.
mod response {
    use std::{cmp::Reverse, collections::BTreeMap};

    use serde::Serialize;

    use crate::db::models;

    use super::format_date;
    use super::CURRENT_PUBLICATION_NAME;

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
        pub messages: HistoricalMessages,
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
    #[derive(Serialize, Debug)]
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
    #[derive(Serialize, Debug, Clone)]
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
        pub fn build(
            active_publication_name: &str,
            active_version: String,
            active_compare_to: Option<String>,
            url: &str,
            publications: &[models::publication::Publication],
            current_publication_name: &str,
            versions: &[Version],
            messages: HistoricalMessages,
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
        fn format_display_date(date: &str, current_date: &str) -> String {
            if date == CURRENT_PUBLICATION_NAME {
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

    /// Messages for the versions endpoint.
    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct HistoricalMessages {
        /// Message for an outdated publication.
        pub publication: Option<String>,
        /// Message for an outdated version.
        pub version: Option<String>,
        /// Message for a comparison between two versions.
        pub comparison: Option<String>,
    }
}

/// Handler for the versions endpoint.
pub async fn versions(
    req: HttpRequest,
    data: web::Data<AppState>,
    params: web::Path<request::Version>,
) -> impl Responder {
    let stele = match get_stele_from_request(&req, data.archive()) {
        Ok(stele) => stele,
        Err(err) => return HttpResponse::BadRequest().body(format!("Error: {err}")),
    };
    let db = data.db();
    let mut publications = publication::Manager::find_all_non_revoked_publications(db, &stele)
        .await
        .unwrap_or_default();

    let Some(current_publication) = publications.first() else {
        return HttpResponse::NotFound().body("No publications found.");
    };

    let mut active_publication_name = params
        .publication
        .clone()
        .unwrap_or_else(|| current_publication.name.clone());

    let active_publication = publications
        .iter()
        .find(|pb| pb.name == active_publication_name);

    let mut url = String::from("/");
    url.push_str(params.path.clone().unwrap_or_default().as_str());

    let mut versions = if let Some(publication) = active_publication {
        publication_versions(db, publication, url.clone()).await
    } else {
        vec![]
    };

    // latest date in active publication
    let current_date = versions
        .first()
        .map_or(String::new(), |ver| ver.date.clone());
    // active version is the version the user is looking at right now
    let mut active_version =
        NaiveDate::parse_from_str(params.date.as_deref().unwrap_or_default(), "%Y-%m-%d")
            .map_or(current_date.clone(), |date| date.clone().to_string());
    let active_compare_to = params.compare_date.clone().map(|date| {
        NaiveDate::parse_from_str(&date, "%Y-%m-%d").map_or_else(
            |_| current_date.clone(),
            |active_date| active_date.to_string(),
        )
    });

    if active_version == current_date {
        active_version = CURRENT_VERSION_DATE.to_owned();
    }

    let messages = historical_messages(
        &versions,
        current_publication,
        &active_publication_name,
        &params.date,
        &active_compare_to,
    );

    if active_publication_name == current_publication.name.clone() {
        active_publication_name = CURRENT_PUBLICATION_NAME.to_owned();
    }

    response::Version::insert_if_not_present(&mut versions, params.date.clone());
    response::Version::insert_if_not_present(&mut versions, active_compare_to.clone());

    let versions_size = versions.len();
    for (idx, version) in versions.iter_mut().enumerate() {
        version.display = format_date(&version.date.clone());
        version.index = versions_size - idx;
    }
    if let Some(ver) = versions.first_mut() {
        ver.display.push_str(" (last modified)");
    };

    let current_version = response::Version::new(
        CURRENT_VERSION_DATE.to_owned(),
        CURRENT_VERSION_NAME.to_owned(),
        versions.first().map_or(0, |ver| ver.index),
    );

    versions.insert(versions_size - current_version.index, current_version);

    let current_publication_name = current_publication.name.clone();
    // duplicate current publication with current label
    publications.insert(
        0,
        Publication::new(
            CURRENT_PUBLICATION_NAME.to_owned(),
            current_publication.date.clone(),
            current_publication.stele.clone(),
        ),
    );

    HttpResponse::Ok().json(response::Versions::build(
        &active_publication_name,
        active_version,
        active_compare_to,
        &url,
        &publications,
        &current_publication_name,
        &versions,
        messages,
    ))
}

/// Get all the versions of a publication.
async fn publication_versions(
    db: &DatabaseConnection,
    publication: &Publication,
    url: String,
) -> Vec<response::Version> {
    let mut versions = vec![];
    let doc_mpath = document_change::Manager::find_doc_mpath_by_url(db, &url)
        .await
        .unwrap_or_default();
    if let Some(mpath) = doc_mpath {
        let doc_versions =
            document_change::Manager::find_all_document_versions_by_mpath_and_publication(
                db,
                &mpath,
                &publication.name,
            )
            .await
            .unwrap_or_default();
        versions = doc_versions.into_iter().map(Into::into).collect();
        return versions;
    }

    let lib_mpath = library_change::Manager::find_lib_mpath_by_url(db, &url)
        .await
        .unwrap_or_default();
    if let Some(mpath) = lib_mpath {
        let coll_versions =
            library_change::Manager::find_all_collection_versions_by_mpath_and_publication(
                db,
                &mpath,
                &publication.name,
            )
            .await
            .unwrap_or_default();
        versions = coll_versions.into_iter().map(Into::into).collect();
    }
    versions
}

/// Extracts the stele from the request.
/// If the `X-Stelae` header is present, it will return the value of the header.
/// Otherwise, it will return the root stele.
fn get_stele_from_request(req: &HttpRequest, archive: &Archive) -> anyhow::Result<String> {
    let req_headers = req.headers();
    let stele = archive.get_root()?.get_qualified_name();

    req_headers.get("X-Stelae").map_or_else(
        || Ok(stele),
        |value| {
            value.to_str().map_or_else(
                |_| anyhow::bail!("Invalid X-Stelae header value"),
                |str| Ok(str.to_owned()),
            )
        },
    )
}

/// Returns historical messages for the versions endpoint.
/// The historical messages currently include:
/// - A message for an outdated publication.
/// - A message for an outdated version.
/// - A message for a comparison between two versions.
fn historical_messages(
    versions: &[response::Version],
    current_publication: &Publication,
    active_publication_name: &str,
    version_date: &Option<String>,
    compare_to_date: &Option<String>,
) -> response::HistoricalMessages {
    let current_publication_name = current_publication.name.as_str();
    let current_version: &str = versions
        .first()
        .map(|lmv| lmv.date.as_str())
        .unwrap_or_default();

    let publication = publication_message(
        active_publication_name,
        current_publication_name,
        current_version,
    );
    let version = version_date.as_ref().and_then(|found_version_date| {
        version_message(
            current_version,
            found_version_date,
            versions,
            compare_to_date,
        )
    });
    let comparison = compare_to_date.as_ref().and_then(|found_compare_to_date| {
        version_date.as_ref().map(|found_version_date| {
            comparison_message(
                found_compare_to_date,
                found_version_date,
                current_version,
                versions,
            )
        })
    });
    response::HistoricalMessages {
        publication,
        version,
        comparison,
    }
}

/// Returns a historical message for an outdated publication.
fn publication_message(
    active_publication_name: &str,
    current_publication_name: &str,
    current_version: &str,
) -> Option<String> {
    if active_publication_name == current_publication_name {
        return None;
    }
    Some(format!(
        "You are viewing a historical publication that was last updated on {current_date} and is no longer being updated.",
        current_date = format_date(current_version)
    ))
}

/// Returns a historical message for an outdated version.
/// Version is outdated if `version_date` is in the past.
fn version_message(
    current_version: &str,
    version_date: &str,
    versions: &[response::Version],
    compare_to_date: &Option<String>,
) -> Option<String> {
    let is_current_version = {
        let current_date =
            NaiveDate::parse_from_str(current_version, "%Y-%m-%d").unwrap_or_default();
        let Ok(parsed_version_date) = NaiveDate::parse_from_str(version_date, "%Y-%m-%d") else {
            return None;
        };
        current_date <= parsed_version_date
    };
    if compare_to_date.is_some() || is_current_version {
        return None;
    }
    let version_date_idx = versions
        .iter()
        .position(|ver| ver.date.as_str() == version_date);
    let (start_date, end_date) = version_date_idx.map_or_else(
        || {
            let end_date = versions
                .iter()
                .find(|ver| ver.date.as_str() > version_date)
                .map(|ver| ver.date.as_str());
            let found_idx = versions
                .iter()
                .position(|ver| ver.date.as_str() == end_date.unwrap_or_default())
                .unwrap_or_default();
            let start_date = versions
                .get(found_idx + 1)
                .map_or_else(|| versions.last(), Some)
                .map(|ver| ver.date.as_str());
            (start_date.unwrap_or_default(), end_date.unwrap_or_default())
        },
        |idx| {
            let start_date = version_date;
            let end_date = versions
                .get(idx - 1)
                .map_or_else(|| versions.first(), Some)
                .map(|ver| ver.date.as_str())
                .unwrap_or_default();
            (start_date, end_date)
        },
    );
    Some(format!("You are viewing this document as it appeared on {version_date}. This version was valid between {start_date} and {end_date}.",
        version_date = format_date(version_date), start_date = format_date(start_date), end_date = format_date(end_date)))
}

/// Returns a historical message for a comparison between two versions.
fn comparison_message(
    compare_to_date: &str,
    version_date: &str,
    current_date: &str,
    versions: &[response::Version],
) -> String {
    let (compare_start_date, compare_end_date) = if version_date > compare_to_date {
        (compare_to_date, version_date)
    } else {
        (version_date, compare_to_date)
    };
    let start_idx = response::Version::find_index_or_closest(versions, compare_start_date);
    let end_idx = response::Version::find_index_or_closest(versions, compare_start_date);
    let num_of_changes = start_idx - end_idx;
    let start_date = format_date(compare_start_date);
    let end_date = if compare_end_date == current_date {
        None
    } else {
        Some(format_date(compare_end_date))
    };
    get_messages_between(num_of_changes, &start_date, end_date)
}

/// Returns a message for the number of changes between two dates.
fn get_messages_between(
    num_of_changes: usize,
    start_date: &str,
    end_date: Option<String>,
) -> String {
    let changes = match num_of_changes {
        0 => "no updates".to_owned(),
        1 => "1 update".to_owned(),
        _ => format!("{num_of_changes} updates"),
    };

    end_date.map_or_else(
        || format!("There have been <strong>{changes}</strong> since {start_date}."),
        |found_end_date| {
            format!(
                "There have been <strong>{changes}</strong> between {start_date} and {found_end_date}."
            )
        },
    )
}

/// Format a date from %Y-%m-%d to %B %d, %Y.
fn format_date(date: &str) -> String {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").map_or(date.to_owned(), |found_date| {
        found_date.format("%B %d, %Y").to_string()
    })
}
