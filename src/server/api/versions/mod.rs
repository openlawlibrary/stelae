//! Handlers for serving historical documents.
#![expect(
    clippy::future_not_send,
    reason = "We don't worry about git2-rs not implementing `Send` trait"
)]
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::NaiveDate;
use std::convert::Into;

use crate::{
    db::{
        models::{
            document_change, document_element, library, library_change,
            publication::{self, Publication},
        },
        DatabaseConnection,
    },
    stelae::archive::Archive,
    utils::paths::clean_path,
};

use self::response::messages;

use super::state::{App as AppState, Global as _};

/// Name of the current publication.
pub const CURRENT_PUBLICATION_NAME: &str = "Current";
/// Name of the current version.
pub const CURRENT_VERSION_NAME: &str = "Current";
/// Date of the current version.
pub const CURRENT_VERSION_DATE: &str = "current";

/// Module that maps the HTTP web request body to structs.
pub mod request;

/// Module that maps the HTTP web response to structs.
pub mod response;

/// Handler for the versions endpoint.
#[tracing::instrument(skip(req, data))]
pub async fn versions(
    req: HttpRequest,
    data: web::Data<AppState>,
    params: web::Path<request::Version>,
) -> impl Responder {
    let stele = match get_stele_from_request(&req, data.archive()) {
        Ok(stele) => stele,
        Err(err) => {
            tracing::error!("Error getting stele from request: {err}");
            return HttpResponse::BadRequest().body(format!("Error: {err}"));
        }
    };
    let db = data.db();
    let mut publications = publication::Manager::find_all_non_revoked_publications(db, &stele)
        .await
        .unwrap_or_default();

    let Some(current_publication) = publications.first() else {
        tracing::warn!("No publications found for stele: {stele}");
        return HttpResponse::NotFound().body("No publications found.");
    };

    let mut active_publication_name = params
        .publication
        .clone()
        .unwrap_or_else(|| current_publication.name.clone())
        .to_lowercase();

    let active_publication = publications
        .iter()
        .find(|pb| pb.name == active_publication_name);

    let url = clean_url_path(&params.path.clone().unwrap_or_default());
    let mut versions = if let Some(publication) = active_publication {
        publication_versions(db, publication, url.clone()).await
    } else if active_publication_name == "current" {
        publication_versions(db, current_publication, url.clone()).await
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
        NaiveDate::parse_from_str(&date, "%Y-%m-%d")
            .map_or_else(|_| date, |active_date| active_date.to_string())
    });

    if active_version == current_date {
        CURRENT_VERSION_DATE.clone_into(&mut active_version);
    }

    let messages = messages::historical(
        &versions,
        current_publication.name.as_str(),
        &active_publication_name,
        &params.date,
        &active_compare_to,
        active_publication.is_some(),
    );

    if active_publication_name == current_publication.name.clone() && params.publication.is_none() {
        CURRENT_PUBLICATION_NAME.clone_into(&mut active_publication_name);
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
    }

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
            current_publication.id.clone(),
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
    tracing::debug!("Fetching publication versions for '{url}'");
    let mut versions = vec![];
    let doc_mpath =
        document_element::Manager::find_doc_mpath_by_url(db, &url, &publication.stele).await;
    if let Ok(mpath) = doc_mpath {
        let doc_versions =
            document_change::Manager::find_all_document_versions_by_mpath_and_publication(
                db,
                &mpath,
                &publication.id,
            )
            .await
            .unwrap_or_default();
        versions = doc_versions.into_iter().map(Into::into).collect();
    } else {
        let lib_mpath = library::Manager::find_lib_mpath_by_url(db, &url, &publication.stele).await;
        if let Ok(mpath) = lib_mpath {
            let coll_versions =
                library_change::Manager::find_all_collection_versions_by_mpath_and_publication(
                    db,
                    &mpath,
                    &publication.id,
                )
                .await
                .unwrap_or_default();
            versions = coll_versions.into_iter().map(Into::into).collect();
        }
    }
    tracing::debug!("Found {} versions", versions.len());
    versions
}

/// Extracts the stele from the request.
/// If the `X-Stelae` header is present, it will return the value of the header.
/// Otherwise, it will return the root stele.
///
/// # Errors
/// Errors if X-Stelae is in invalid format
pub fn get_stele_from_request(req: &HttpRequest, archive: &Archive) -> anyhow::Result<String> {
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

/// Format a date from %Y-%m-%d to %B %d, %Y.
fn format_date(date: &str) -> String {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").map_or(date.to_owned(), |found_date| {
        found_date.format("%B %d, %Y").to_string()
    })
}

/// Clean the url path by removing the trailing slash.
fn clean_url_path(path: &str) -> String {
    let mut url = String::from('/');
    let url_parts = clean_path(path);
    url.push_str(&url_parts);
    url
}
