//! API endpoint for serving **historical (versioned)** git blobs.
///
/// The `_date` endpoint provides access to file contents as they existed on a
/// specific version date. It allows clients to retrieve historical document
/// states rather than only the latest revision.
///
/// # Overview
/// - Resolves the repository and file path from the request.
/// - Maps the provided version date to the corresponding commit in the data
///   repository.
/// - Retrieves the file blob from that commit.
/// - Rewrites internal document references (HTML/JSON) so links point to the
///   same historical date context.
///
/// # Behavior
/// - If the requested date matches the current version, the latest content is served.
/// - If no commit exists for the given date, the closest prior version is used.
/// - Content type is inferred from the file path.
///
/// # Example
/// ```http
/// GET /_date/2025-03-04/org/repo/path/to/document.html
/// ```
pub mod request;
pub mod notifications;

use super::doc_transform::{
    build_absolute_url, build_url_prefix, format_date_display, get_doc_version_dates,
    get_version_start_end_current, insert_notification, update_doc_urls, update_json_content,
};
use super::state::App as AppState;
use crate::server::api::versions::get_stele_from_request;
use crate::utils;
use crate::utils::archive::get_name_parts;
use crate::{
    db::{
        models::{
            data_repo_commits::{self},
            publication::{self, Publication},
        },
        DatabaseTransaction,
    },
    utils::{git::Repo, http::get_contenttype, paths::clean_path},
};
use actix_web::http::header::ContentType;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use anyhow::anyhow;
use mime::{APPLICATION, HTML, JSON, TEXT};
use std::path::Path;

/// Handler for the date endpoint.
#[tracing::instrument(skip(req, data))]
#[expect(
    clippy::future_not_send,
    reason = "We don't worry about git2-rs not implementing `Send` trait"
)]
#[expect(
    clippy::too_many_lines,
    reason = "Line count is not a problem here; the function is readable as a single sequential flow"
)]
pub async fn date(
    req: HttpRequest,
    data: web::Data<AppState>,
    params: web::Path<request::Date>,
) -> impl Responder {
    let tx_res = data.db.pool.begin().await;
    let mut tx = match tx_res {
        Ok(tx_inner) => DatabaseTransaction { tx: tx_inner },
        Err(err) => {
            tracing::error!(error = %err, "Couldn't initialize Database Transaction");
            return HttpResponse::NotFound().body("");
        }
    };
    // get publication and repo name
    let auth_stele_name = match get_stele_from_request(&req, &data.archive) {
        Ok(rn) => rn,
        Err(err) => {
            tracing::error!(error = %err, "Couldn't extract auth_repo_name from request header");
            return HttpResponse::NotFound().body("");
        }
    };
    let publication = match get_publication_by_name_or_latest(
        &mut tx,
        &auth_stele_name,
        params.pub_name.as_deref(),
    )
    .await
    {
        Ok(publication) => publication,
        Err(err) => {
            tracing::error!(error = %err, "Couldn't find publication for {auth_stele_name} (pub_name: {:?})", params.pub_name.clone());
            return HttpResponse::NotFound().body("");
        }
    };
    let html_repo_name = match get_html_repo(&data, &auth_stele_name) {
        Ok(html_repo) => html_repo,
        Err(err) => {
            tracing::error!(error = %err, "Couldn't find html repo for {auth_stele_name}");
            return HttpResponse::NotFound().body("");
        }
    };

    let commit = match get_commit(
        &mut tx,
        publication.id.as_str(),
        params.version_date.clone().unwrap_or_default().as_str(),
    )
    .await
    {
        Ok(commit) => commit,
        Err(err) => {
            let error_msg = format!(
                "Couldn't find commit for publication {} and version_date {}",
                publication.id,
                params.version_date.clone().unwrap_or_default()
            );
            tracing::error!(error = %err, error_msg);
            return HttpResponse::NotFound().body("");
        }
    };

    let doc_path = params.path.as_deref().unwrap_or("");
    let (blob, content_type) = match get_document(
        &data.archive.path,
        &html_repo_name,
        doc_path,
        commit.as_str(),
    ) {
        Ok(val) => val,
        Err(err) => {
            tracing::error!(error = %err, "Couldn't find file: repo_name: {html_repo_name} | doc_path: {doc_path}| commit {commit}");
            return HttpResponse::NotFound().body("");
        }
    };
    let doc_str = match str::from_utf8(&blob.content) {
        Ok(dc) => dc,
        Err(err) => {
            tracing::error!(error = %err, "Couldn't convert blob to utf8");
            return HttpResponse::NotFound().body("");
        }
    };

    let mime = &content_type.0;
    #[expect(
        clippy::else_if_without_else,
        reason = "Only specific values are handled; all other cases intentionally do nothing"
    )]
    if mime.type_() == TEXT && mime.subtype() == HTML {
        let version_date = params.version_date.clone().unwrap_or_default();
        let pub_name = params.pub_name.as_deref().unwrap_or_default();

        let mut doc = match update_doc_urls(doc_str, doc_path, &version_date, pub_name) {
            Ok(updated) => updated,
            Err(err) => {
                tracing::error!(error = %err, "Couldn't update html");
                doc_str.to_owned()
            }
        };

        let version_dates = get_doc_version_dates(&data.db, &publication, doc_path).await;
        let (version_start_date, version_end_date, current_date) =
            get_version_start_end_current(&version_dates, &version_date);

        // Outdated document notification: shown when a newer version exists
        if !version_date.is_empty() {
            if let Some(end_date) = version_end_date {
                let current_date_str = current_date.as_deref().unwrap_or_default();
                let url_prefix = build_url_prefix(pub_name, current_date_str);
                let current_version_url =
                    build_absolute_url(&req, &format!("{url_prefix}/{doc_path}"));
                let notification = notifications::outdated_doc(
                    &format_date_display(&version_date),
                    &format_date_display(version_start_date.as_deref().unwrap_or_default()),
                    &format_date_display(&end_date),
                    &current_version_url,
                );
                doc = insert_notification(&doc, &notification);
            }
        }

        // Outdated publication notification: shown when a newer publication exists
        match get_publication(&mut tx, &auth_stele_name).await {
            Ok(latest_publication) => {
                if publication.name != latest_publication.name {
                    let current_date_str = current_date.as_deref().unwrap_or_default();
                    let url_prefix = build_url_prefix(&latest_publication.name, &version_date);
                    let current_pub_url =
                        build_absolute_url(&req, &format!("{url_prefix}/{doc_path}"));
                    let notification = notifications::outdated_pub(
                        &format_date_display(current_date_str),
                        &current_pub_url,
                    );
                    doc = insert_notification(&doc, &notification);
                }
            }
            Err(err) => {
                tracing::warn!(error = %err, "Couldn't find latest publication for notification");
            }
        }

        return HttpResponse::Ok()
            .insert_header(content_type)
            .body(doc.into_bytes());
    } else if mime.type_() == APPLICATION && mime.subtype() == JSON {
        let mod_doc = update_json_content(
            doc_str,
            doc_path,
            params.version_date.clone().unwrap_or_default().as_str(),
            params.pub_name.as_deref().unwrap_or_default(),
        );
        let content = mod_doc.as_bytes().to_vec();
        return HttpResponse::Ok().insert_header(content_type).body(content);
    }
    return HttpResponse::Ok()
        .insert_header(content_type)
        .body(blob.content);
}

/// Finds and returns the HTML repository name for the given stelae.
/// # Errors
///
/// Returns an error if the stele cannot be found, no repositories are defined,
/// or no HTML repository exists for the given repository name.
#[expect(clippy::pattern_type_mismatch, reason = "..")]
pub fn get_html_repo(data: &web::Data<AppState>, repo_name: &str) -> anyhow::Result<String> {
    let stelae = data.archive.get_stelae();
    let Some((_, auth_repo)) = stelae.iter().find(|(s_name, _)| s_name == repo_name) else {
        return Err(anyhow::anyhow!(
            "Repository '{}' not found in stelae",
            repo_name
        ));
    };

    if let Some(repositories) = auth_repo.repositories.as_ref() {
        if let Some((key, _)) = repositories
            .repositories
            .iter()
            .find(|(_, repo)| repo.custom.repository_type == Some("html".to_owned()))
        {
            return Ok(key.to_owned());
        }
    }

    Err(anyhow::anyhow!(
        "No html repository in '{}' stelae",
        repo_name
    ))
}

/// Retrieves the commit hash for a given publication ID and version date.
///
/// # Arguments
///
/// * `tx` - A mutable reference to the `DatabaseTransaction`.
/// * `publication_id` - The ID of the publication to query.
/// * `version_date` - The version date of the publication, as a string.
///
/// # Returns
///
/// Returns the commit hash as a `String` if found.
///
/// # Errors
///
/// Returns an error if the database query fails or the commit cannot be found.
pub async fn get_commit(
    tx: &mut DatabaseTransaction,
    publication_id: &str,
    version_date: &str,
) -> anyhow::Result<String> {
    //
    let commit = data_repo_commits::TxManager::find_commit_by_pub_id_and_version_date(
        tx,
        publication_id,
        version_date,
    )
    .await?;
    Ok(commit.commit_hash)
}

/// Retrieves a document blob and its detected content type from a repository.
///
/// The document is resolved from the given repository path at the specified
/// commit. If no commit is provided, `HEAD` is used by default.
///
/// # Arguments
///
/// * `archive_path` - Path to the local archive containing repositories
/// * `repo_name` - Repository identifier in `<namespace>/<org>` format
/// * `path` - Path to the document inside the repository
/// * `commit` - Optional commit SHA; defaults to `HEAD` if `None`
///
/// # Returns
///
/// Returns a tuple containing:
/// * `Blob` - The resolved Git blob for the requested document
/// * `ContentType` - The inferred HTTP content type of the document
///
/// # Errors
///
/// Returns an error if:
/// * The repository name cannot be parsed
/// * The blob cannot be found at the given path or commit
/// * Any underlying repository operation fails
pub fn get_document(
    archive_path: &Path,
    repo_name: &str,
    path: &str,
    commit: &str,
) -> anyhow::Result<(utils::git::Blob, ContentType)> {
    let (namespace, org) = get_name_parts(repo_name)?;
    let blob = Repo::find_blob(archive_path, &namespace, &org, path, commit)?;
    let blob_path = clean_path(path);
    let contenttype = get_contenttype(&blob_path);
    Ok((blob, contenttype))
}

/// Finds the most recent non-revoked publication for the given `stelae_name`.
///
/// # Arguments
///
/// * `tx` - Active database transaction
/// * `stelae_name` - Publication (stelae) identifier
///
/// # Errors
///
/// Returns an error if the database query fails or the publication cannot be found.
pub async fn get_publication(
    tx: &mut DatabaseTransaction,
    stelae_name: &str,
) -> anyhow::Result<Publication> {
    let Some(publication) = publication::TxManager::find_last_inserted(tx, stelae_name).await?
    else {
        return Err(anyhow!("No publication found for {stelae_name}"));
    };

    Ok(publication)
}

/// Finds a publication by name if provided, falling back to the latest non-revoked publication.
///
/// If `pub_name` is `Some`, attempts to find a non-revoked publication matching that name
/// for the given stelae. If none is found (or `pub_name` is `None`), returns the most
/// recently inserted non-revoked publication.
///
/// # Arguments
///
/// * `tx` - Active database transaction
/// * `stelae_name` - Publication (stelae) identifier
/// * `pub_name` - Optional publication name to look up first
///
/// # Errors
///
/// Returns an error if the database query fails or no publication can be found.
pub async fn get_publication_by_name_or_latest(
    tx: &mut DatabaseTransaction,
    stelae_name: &str,
    pub_name: Option<&str>,
) -> anyhow::Result<Publication> {
    if let Some(name) = pub_name {
        if let Some(publication) =
            publication::TxManager::find_by_name_and_stele(tx, name, stelae_name).await?
        {
            return Ok(publication);
        }
        tracing::info!("Couldn't find publication for pub_name: {:?})", pub_name);
    }
    get_publication(tx, stelae_name).await
}
