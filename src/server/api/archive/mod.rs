//! API endpoint for serving git blobs.
/// The `_archive` endpoint provides access to the contents of files stored within
/// repositories managed by stelae.
///
/// This endpoint only permits access to files that belong to **public** repositories.
/// Attempts to access files in private or restricted repositories will result in a
/// `403 Forbidden` response.
///
/// # Overview
/// - Resolves repositories and files based on the provided path and query parameters.
/// - Verifies that the target repository is publicly accessible.
/// - Returns the requested file content if access is allowed.
///
/// # Restrictions
/// - Only public repositories are accessible.
/// - Authorization headers or guards may further restrict access.
///
/// # Example
/// ```http
/// GET /_archive/org/repo?path=/README.md&commitish=main
/// ```
use std::path::PathBuf;
use std::sync::Arc;

use crate::server::headers;
use crate::utils::archive::get_name_parts;
use crate::utils::git::{Repo, GIT_REQUEST_NOT_FOUND};
use crate::utils::http::get_contenttype;
use crate::utils::paths::clean_path;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use git2::{self, ErrorCode};
use request::ArchiveQueryData;

use super::super::errors::HTTPError;
use super::state::Global;
use super::versions::get_stele_from_request;

/// Module that maps the HTTP web request body to structs.
pub mod request;

#[expect(
    clippy::pattern_type_mismatch,
    reason = "The pattern is clear and intentional; matching by reference adds unnecessary verbosity for this context."
)]
/// Return the content in the stelae archive in the `{namespace}/{name}`
/// repo at the `commitish` commit at the `path` path.
/// Return 404 if any are not found or there are any errors.
#[tracing::instrument(name = "Retrieving a Git blob", skip(path, data, query))]
#[expect(
    clippy::future_not_send,
    reason = "We don't worry about git2-rs not implementing `Send` trait"
)]
pub async fn get_blob(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    query: web::Query<ArchiveQueryData>,
    data: web::Data<PathBuf>,
    archive_data: web::Data<Arc<dyn Global>>,
    is_guarded: web::Data<bool>,
) -> impl Responder {
    let stele_name = match get_stele_from_request(&req, archive_data.archive()) {
        Ok(stele) => stele,
        Err(err) => {
            tracing::error!("Error getting stele from request: {err}");
            return HttpResponse::BadRequest().body(format!("Error: {err}"));
        }
    };
    let (org, _) = match get_name_parts(&stele_name) {
        Ok(parts) => parts,
        Err(err) => return HttpResponse::BadRequest().body(format!("Error: {err}")),
    };
    let (namespace, name) = path.into_inner();
    let Ok(root_stele) = archive_data.archive().get_root() else {
        return HttpResponse::NotFound().body("Can not find root stele in archive");
    };
    if **is_guarded && namespace == org {
        match root_stele.get_repositories_for_commitish("HEAD") {
            Ok(Some(repos))
                if repos
                    .repositories
                    .contains_key(&format!("{namespace}/{name}")) =>
            {
                return HttpResponse::Forbidden().body("Forbidden repository");
            }
            Ok(Some(_) | None) => {
                tracing::warn!("No matching repo found or root repositories empty");
            }
            Err(err) => {
                tracing::error!("Error fetching root repositories: {err}");
                return HttpResponse::BadRequest().body(format!("Error: {err}"));
            }
        }
    };
    let query_data: ArchiveQueryData = query.into_inner();
    let commitish = query_data.commitish.unwrap_or_else(|| String::from("HEAD"));
    let file_path = query_data.path.unwrap_or_default();
    let stelae = archive_data.archive().get_stelae();
    let Some((_, stele)) = stelae
        .iter()
        .find(|(s_name, _)| *s_name == format!("{namespace}/law"))
    else {
        return HttpResponse::BadRequest().body("Can not find stele in archive stelae");
    };
    let repositories = match stele.get_repositories_for_commitish("HEAD") {
        Ok(Some(repos)) => repos,
        Ok(None) => {
            tracing::error!("No repositories found");
            return HttpResponse::BadRequest().body("No repositories found");
        }
        Err(err) => {
            tracing::error!("Error fetching repositories: {err}");
            return HttpResponse::BadRequest().body(format!("Error: {err}"));
        }
    };
    let full_name = format!("{namespace}/{name}");
    if !repositories.repositories.contains_key(&full_name) {
        return HttpResponse::BadRequest()
            .body("Repository is not in list of allowed repositories");
    }
    let archive_path = &data;
    let blob = Repo::find_blob(archive_path, &namespace, &name, &file_path, &commitish);
    let blob_path = clean_path(&file_path);
    let contenttype = get_contenttype(&blob_path);
    match blob {
        Ok(found_blob) => {
            let content = found_blob.content;
            let filepath = found_blob.path;
            HttpResponse::Ok()
                .insert_header(contenttype)
                .insert_header((headers::HTTP_X_FILE_PATH, filepath))
                .body(content)
        }
        Err(error) => blob_error_response(&error, &namespace, &name),
    }
}

/// A centralised place to match potentially unsafe internal errors to safe user-facing error responses
#[expect(clippy::wildcard_enum_match_arm, reason = "Allows _ for enum matching")]
#[tracing::instrument(name = "Error with Git blob request", skip(error, namespace, name))]
fn blob_error_response(error: &anyhow::Error, namespace: &str, name: &str) -> HttpResponse {
    tracing::error!("{error}",);
    if let Some(git_error) = error.downcast_ref::<git2::Error>() {
        return match git_error.code() {
            // TODO: check this is the right error
            ErrorCode::NotFound => {
                HttpResponse::NotFound().body(format!("repo {namespace}/{name} does not exist"))
            }
            _ => HttpResponse::InternalServerError().body("Unexpected Git error"),
        };
    }
    match error {
        // TODO: Obviously it's better to use custom `Error` types
        _ if error.to_string() == GIT_REQUEST_NOT_FOUND => {
            HttpResponse::NotFound().body(HTTPError::NotFound.to_string())
        }
        _ => HttpResponse::InternalServerError().body(HTTPError::InternalServerError.to_string()),
    }
}
