//! API endpoint for serving current documents from Stele repositories.
#![allow(clippy::infinite_loop)]
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use lazy_static::lazy_static;
use regex::Regex;

use crate::utils::{git::Repo, http::get_contenttype};

use super::state::{RepoData as RepoState, Shared as SharedState};
/// Most-recent git commit
const HEAD_COMMIT: &str = "HEAD";

#[allow(clippy::expect_used)]
/// Remove leading and trailing `/`s from the `path` string.
fn clean_path(path: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new("(?:^/*|/*$)").expect("Failed to compile regex!?!");
    }
    RE.replace_all(path, "").to_string()
}

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
    let blob = find_current_blob(&data.repo, &shared, &path);
    match blob {
        Ok(content) => HttpResponse::Ok().insert_header(contenttype).body(content),
        Err(error) => {
            tracing::debug!("{path}: {error}",);
            HttpResponse::BadRequest().into()
        }
    }
}

/// Find the latest blob for the given path from the given repo
/// Latest blob is found by looking at the HEAD commit
#[allow(clippy::panic_in_result_fn, clippy::unreachable)]
#[tracing::instrument(name = "Finding document", skip(repo, shared))]
fn find_current_blob(repo: &Repo, shared: &SharedState, path: &str) -> anyhow::Result<Vec<u8>> {
    let blob = repo.get_bytes_at_path(HEAD_COMMIT, path);
    match blob {
        Ok(content) => Ok(content),
        Err(error) => {
            if let Some(fallback) = shared.fallback.as_ref() {
                let fallback_blob = fallback.repo.get_bytes_at_path(HEAD_COMMIT, path);
                return fallback_blob.map_or_else(
                    |err| anyhow::bail!("No fallback blob found - {}", err.to_string()),
                    Ok,
                );
            }
            anyhow::bail!("No fallback repo - {}", error.to_string())
        }
    }
}
