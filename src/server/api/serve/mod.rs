//! API endpoint for serving current documents from Stele repositories.
#![allow(clippy::infinite_loop)]
use actix_web::{web, HttpRequest, HttpResponse, Responder};

use crate::{
    server::errors::HTTPError,
    utils::{git::Repo, http::get_contenttype, paths::clean_path},
};

use super::state::{RepoData as RepoState, Shared as SharedState};
/// Most-recent git commit
const HEAD_COMMIT: &str = "HEAD";

/// Serve current document
#[allow(clippy::future_not_send)]
pub async fn serve(
    req: HttpRequest,
    shared: web::Data<SharedState>,
    data: web::Data<RepoState>,
) -> impl Responder {
    let prefix = req
        .match_info()
        .get("prefix")
        .unwrap_or_default()
        .to_owned();
    let tail = req.match_info().get("tail").unwrap_or_default().to_owned();
    let mut path = format!("{prefix}/{tail}");
    path = clean_path(&path);
    let contenttype = get_contenttype(&path);
    let blob = find_current_blob(&data, &shared, &path);
    match blob {
        Ok(content) => HttpResponse::Ok().insert_header(contenttype).body(content),
        Err(error) => {
            tracing::debug!("{path}: {error}",);
            HttpResponse::NotFound().body(HTTPError::NotFound.to_string())
        }
    }
}

/// Find the latest blob for the given path from the given repo
/// Latest blob is found by looking at the HEAD commit
#[allow(clippy::panic_in_result_fn, clippy::unreachable)]
#[tracing::instrument(name = "Finding document", skip(repo, shared))]
fn find_current_blob(
    repo: &RepoState,
    shared: &SharedState,
    path: &str,
) -> anyhow::Result<Vec<u8>> {
    let blob = Repo::find_blob(&repo.archive_path, &repo.org, &repo.name, path, HEAD_COMMIT);
    match blob {
        Ok(content) => Ok(content),
        Err(error) => {
            if let Some(fallback) = shared.fallback.as_ref() {
                let fallback_blob = Repo::find_blob(
                    &fallback.archive_path,
                    &fallback.org,
                    &fallback.name,
                    path,
                    HEAD_COMMIT,
                );
                return fallback_blob.map_or_else(
                    |err| anyhow::bail!("No fallback blob found - {}", err.to_string()),
                    Ok,
                );
            }
            anyhow::bail!("No fallback repo - {}", error.to_string())
        }
    }
}
