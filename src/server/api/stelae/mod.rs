//! API endpoint for serving git blobs.

use std::sync::Arc;

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use request::StelaeQueryData;

use super::state::Global;
use crate::utils::git::{Repo, GIT_REQUEST_NOT_FOUND};
use crate::utils::http::get_contenttype;
use crate::utils::paths::clean_path;
use git2::{self, ErrorCode};

use super::super::errors::HTTPError;

/// Module that maps the HTTP web request body to structs.
pub mod request;

/// Return the content in the stelae archive in the `{namespace}/{name}`
/// repo at the `commitish` commit at the `remainder` path.
/// Return 404 if any are not found or there are any errors.
#[tracing::instrument(name = "Retrieving a Git blob", skip(path, data, query))]
#[expect(
    clippy::future_not_send,
    reason = "We don't worry about git2-rs not implementing `Send` trait"
)]
pub async fn get_blob(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    query: web::Query<StelaeQueryData>,
    data: web::Data<Arc<dyn Global>>,
) -> impl Responder {
    let (namespace, name) = path.into_inner();
    let query_data: StelaeQueryData = query.into_inner();
    let commitish = query_data.commitish.unwrap_or_default();
    let remainder = query_data.remainder.unwrap_or_default();
    let archive_path = &data.archive().path.clone();
    let blob = Repo::find_blob(archive_path, &namespace, &name, &remainder, &commitish);
    let blob_path = clean_path(&remainder);
    let contenttype = get_contenttype(&blob_path);
    match blob {
        Ok(content) => HttpResponse::Ok().insert_header(contenttype).body(content),
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
