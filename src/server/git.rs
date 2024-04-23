//! Legacy git microserver.

#![allow(
    // Unused asyncs are the norm in Actix route definition files
    clippy::unused_async,
    clippy::unreachable,
    clippy::let_with_type_underscore,
    // Clippy wrongly detects the `infinite_loop` lint on functions with tracing::instrument!
    clippy::infinite_loop
)]

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use git2::{self, ErrorCode};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    io,
    path::{Path, PathBuf},
};
use tracing_actix_web::TracingLogger;

use super::errors::StelaeError;
use crate::server::tracing::StelaeRootSpanBuilder;
use crate::utils::git::{Repo, GIT_REQUEST_NOT_FOUND};
use crate::utils::http::get_contenttype;

/// Global, read-only state passed into the actix app
struct AppState {
    /// path to the Stelae archive
    archive_path: PathBuf,
}

#[allow(clippy::expect_used)]
/// Remove leading and trailing `/`s from the `path` string.
fn clean_path(path: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new("(?:^/*|/*$)").expect("Failed to compile regex!?!");
    }
    RE.replace_all(path, "").to_string()
}

/// Root index path
#[get("/")]
async fn index() -> &'static str {
    "Welcome to Stelae"
}

/// Just for development purposes at the moment
#[get("{path}")]
async fn misc(path: web::Path<String>) -> actix_web::Result<&'static str, StelaeError> {
    match path.as_str() {
        "error" => Err(StelaeError::GitError),
        _ => Ok("\u{2728}"),
    }
}

/// Return the content in the stelae archive in the `{namespace}/{name}`
/// repo at the `commitish` commit at the `remainder` path.
/// Return 404 if any are not found or there are any errors.
#[get("/{namespace}/{name}/{commitish}{remainder:/+([^{}]*?)?/*}")]
#[tracing::instrument(name = "Retrieving a Git blob", skip(path, data))]
async fn get_blob(
    path: web::Path<(String, String, String, String)>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (namespace, name, commitish, remainder) = path.into_inner();
    let archive_path = &data.archive_path;
    let blob = find_blob(archive_path, &namespace, &name, &remainder, &commitish);
    let blob_path = clean_path(&remainder);
    let contenttype = get_contenttype(&blob_path);
    match blob {
        Ok(content) => HttpResponse::Ok().insert_header(contenttype).body(content),
        Err(error) => blob_error_response(&error, &namespace, &name),
    }
}

/// Do the work of looking for the requested Git object.
// TODO: This, and `clean_path`, look like they could live in `utils::git::Repo`
fn find_blob(
    archive_path: &Path,
    namespace: &str,
    name: &str,
    remainder: &str,
    commitish: &str,
) -> anyhow::Result<Vec<u8>> {
    let repo = Repo::new(archive_path, namespace, name)?;
    let blob_path = clean_path(remainder);
    let blob = repo.get_bytes_at_path(commitish, &blob_path)?;
    Ok(blob)
}

/// A centralised place to match potentially unsafe internal errors to safe user-facing error responses
#[allow(clippy::wildcard_enum_match_arm)]
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
            HttpResponse::NotFound().body(GIT_REQUEST_NOT_FOUND)
        }
        _ => HttpResponse::InternalServerError().body("Unexpected server error"),
    }
}

/// Serve git repositories in the Stelae archive.
#[actix_web::main] // or #[tokio::main]
pub async fn serve_git(raw_archive_path: &str, archive_path: PathBuf, port: u16) -> io::Result<()> {
    let bind = "127.0.0.1";
    let message = "Serving content from the Stelae archive at";
    tracing::info!("{message} '{raw_archive_path}' on http://{bind}:{port}.",);

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::<StelaeRootSpanBuilder>::new())
            .service(index)
            .service(misc)
            .service(get_blob)
            .app_data(web::Data::new(AppState {
                archive_path: archive_path.clone(),
            }))
    })
    .bind((bind, port))?
    .run()
    .await
}
