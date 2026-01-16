//! API endpoint for serving git blobs.
/// The `_archive` endpoint provides access to the contents of files stored within
/// repositories.
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
pub mod request;

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
use chrono::NaiveDate;
use libxml::parser::Parser;
use libxml::tree::SaveOptions;
use libxml::tree::{Document, Node, NodeType};
use libxml::xpath::Context;
use mime::{APPLICATION, HTML, JSON, TEXT};
use regex::Regex;
use serde_json::Value;
use std::path::Path;
use url::Url;

/// Handler for the date endpoint.
#[tracing::instrument(skip(req, data))]
#[expect(
    clippy::future_not_send,
    reason = "We don't worry about git2-rs not implementing `Send` trait"
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
    let publication = match get_publication(&mut tx, &auth_stele_name).await {
        Ok(publication) => publication,
        Err(err) => {
            tracing::error!(error = %err, "Couldn't find latest publication for {auth_stele_name}");
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
        let mod_doc = update_doc_urls(
            doc_str,
            doc_path,
            params.version_date.clone().unwrap_or_default().as_str(),
        );

        let content = match mod_doc {
            Ok(con) => con.as_bytes().to_vec(),
            Err(err) => {
                tracing::error!(error = %err, "Couldn't update html");
                blob.content
            }
        };
        return HttpResponse::Ok().insert_header(content_type).body(content);
    } else if mime.type_() == APPLICATION && mime.subtype() == JSON {
        let mod_doc = update_json_content(
            doc_str,
            doc_path,
            params.version_date.clone().unwrap_or_default().as_str(),
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

/// Updates document URLs to point to a historical (versioned) prefix.
///
/// The function parses the provided HTML document, walks through relevant
/// elements (`meta`, `a`, `span`, headings, etc.), and rewrites selected
/// attributes so that local URLs are prefixed with the given `_date/version_date`.
/// It also injects a `<meta>` tag describing the historical prefix.
///
/// # Arguments
///
/// * `html_doc_str` – HTML document content as a UTF-8 string
/// * `path_str` – Request path used to resolve and normalize relative URLs
/// * `version_date` – Date used to construct the historical URL prefix
///
/// # Returns
///
/// Returns the updated HTML document as a string.
///
/// # Errors
///
/// Returns an error if the HTML cannot be parsed or if a required regular
/// expression fails to compile.
#[expect(
    clippy::else_if_without_else,
    reason = "Only specific values are handled; all other cases intentionally do nothing"
)]
pub fn update_doc_urls(
    html_doc_str: &str,
    path_str: &str,
    version_date: &str,
) -> anyhow::Result<String> {
    let path = if path_str.starts_with('/') {
        path_str.to_owned()
    } else {
        format!("/{path_str}")
    };
    let versioned_url = format!("/_date/{version_date}");
    let version_date_fmt = NaiveDate::parse_from_str(version_date, "%Y-%m-%d").map_or_else(
        |_| version_date.to_owned(),
        |date| date.format("%B %d, %Y").to_string(),
    );
    let versioned_title = format!("Historical version from {version_date_fmt}");
    let heading_re = Regex::new(r"^h\d$")?;

    let parser = Parser::default();
    let doc: Document = parser.parse_string(html_doc_str)?;
    let ctx =
        Context::new(&doc).map_err(|err| anyhow::anyhow!("Failed to create Context: {:?}", err))?;

    let nodes = ctx
        .evaluate("//*")
        .map_err(|err| anyhow::anyhow!("Failed to evaluate XPath '//*': {:?}", err))?
        .get_nodes_as_vec();

    for mut node in nodes {
        let tag = node.get_name();

        match tag.as_str() {
            "meta" => {
                if let Some(prop) = node.get_attribute("property") {
                    if prop == "og:title" {
                        if let Some(content) = node.get_attribute("content") {
                            node.set_attribute(
                                "content",
                                &format!("{content} | {versioned_title}"),
                            )
                            .map_err(|_err| anyhow::anyhow!("Failed to update attribute"))?;
                        }
                    } else if prop == "og:url" {
                        if let Some(content) = node.get_attribute("content") {
                            let updated = content.replace(&path, &format!("{versioned_url}{path}"));
                            node.set_attribute("content", &updated)
                                .map_err(|_err| anyhow::anyhow!("Failed to update attribute"))?;
                        }
                    }
                }

                if let Some(itemprop) = node.get_attribute("itemprop") {
                    if itemprop == "full-html" || itemprop == "toc-json" {
                        update_el_attr(&mut node, "content", &path, &versioned_url);
                    }
                }
            }

            "a" => update_el_attr(&mut node, "href", &path, &versioned_url),

            "span" => update_el_attr(&mut node, "id", &path, &versioned_url),

            "object" => {
                if node.get_attribute("type").as_deref() == Some("application/pdf") {
                    update_el_attr(&mut node, "data", &path, &versioned_url);
                }
            }

            _ if heading_re.is_match(&tag) => {
                update_el_attr(&mut node, "id", &path, &versioned_url);
            }

            _ => {}
        }
    }

    // Insert historical-prefix meta tag
    let head = ctx
        .evaluate("/html/head")
        .map_err(|_err| anyhow::anyhow!("Failed to update attribute"))?
        .get_nodes_as_vec()
        .into_iter()
        .next();

    add_historical_meta_tag(head, &doc, &versioned_url)?;

    let options = SaveOptions {
        no_declaration: true,
        ..Default::default()
    };
    Ok(doc.to_string_with_options(options))
}

/// Adds a `<meta>` tag with the `itemprop="historical-prefix"` attribute to the
/// document `<head>`.
///
/// The meta tag’s `content` attribute is set to the provided historical URL
/// prefix (typically a versioned `/_date/...` path). If a `<head>` element
/// exists, the tag is inserted at a deterministic position among its children (The element is inserted at index 13 to align with existing meta tag grouping).
/// If no `<head>` is found, the document is left unchanged.
///
/// # Arguments
///
/// * `doc` - Parsed HTML document to be modified
/// * `versioned_url` - Historical URL prefix to store in the meta tag
///
/// # Errors
///
/// Returns an error if DOM manipulation fails, such as when creating nodes,
/// setting attributes, or inserting the meta element.
fn add_historical_meta_tag(
    head_node: Option<Node>,
    doc: &Document,
    versioned_url: &str,
) -> anyhow::Result<(), anyhow::Error> {
    if let Some(mut head) = head_node {
        let mut meta = Node::new("meta", None, doc)
            .map_err(|err| anyhow::anyhow!("Failed to creat 'meta' node: {:?}", err))?;
        meta.set_attribute("itemprop", "historical-prefix")
            .map_err(|err| anyhow::anyhow!("Failed to update attribute 'itemprop': {err}"))?;
        meta.set_attribute("content", versioned_url)
            .map_err(|err| anyhow::anyhow!("Failed to update attribute 'content': {err}"))?;

        let mut element_children: Vec<Node> = head
            .get_child_nodes()
            .into_iter()
            .filter(|n| {
                n.get_type() != Some(NodeType::TextNode) || !n.get_content().trim().is_empty()
            })
            .collect();

        if let Some(ref_node) = element_children.get_mut(13) {
            // Used for formating indentations
            let mut newline = Node::new_text("\n    ", doc)
                .map_err(|err| anyhow::anyhow!("Failed to create text node: {:?}", err))?;
            ref_node
                .add_prev_sibling(&mut meta)
                .map_err(|err| anyhow::anyhow!("Failed to add_prev_sibling 'meta': {err}"))?;
            ref_node
                .add_prev_sibling(&mut newline)
                .map_err(|err| anyhow::anyhow!("Failed to add_prev_sibling 'text': {err}"))?;
        } else {
            // Used for formating indentations
            let mut newline = Node::new_text("\n  ", doc)
                .map_err(|err| anyhow::anyhow!("Failed to create text node: {:?}", err))?;
            let mut newspaces = Node::new_text("  ", doc)
                .map_err(|err| anyhow::anyhow!("Failed to create text node: {:?}", err))?;
            head.add_child(&mut newspaces)
                .map_err(|err| anyhow::anyhow!("Failed to aooend 'text' node to 'head': {err}"))?;
            head.add_child(&mut meta)
                .map_err(|err| anyhow::anyhow!("Failed to append 'meta' node to 'head': {err}"))?;
            head.add_child(&mut newline)
                .map_err(|err| anyhow::anyhow!("Failed to append 'text' node to 'head': {err}"))?;
        }
    }

    Ok(())
}

/// Updates a single HTML element attribute by rewriting local URLs.
///
/// If the attribute value ends with `.pdf`, the URL is resolved against the
/// normalized document path. Otherwise, local URLs starting with `/` are
/// prefixed with the provided historical version URL.
///
/// # Arguments
///
/// * `node` - Mutable reference to the xml node
/// * `attr` - Attribute name to update (e.g. `href`, `src`, `content`)
/// * `path` - Current document path, used for resolving relative URLs
/// * `versioned_url` - Prefix added to local URLs for historical versions
///
#[expect(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "File extension comparison is intentionally case-sensitive; only lowercase `.pdf` URLs are expecte"
)]
pub fn update_el_attr(node: &mut Node, attr: &str, path: &str, versioned_url: &str) {
    let Some(value) = node.get_attribute(attr) else {
        return;
    };

    let new_value = if value.ends_with(".pdf") {
        let base = path
            .replace("/index.html", "")
            .replace("/index.full.html", "")
            .trim_end_matches('/')
            .to_owned();
        // Url:parse return Err on invalid url - so for paths we add dummy schema/base
        if let Ok(base_url) = Url::parse(&format!("http://oll.dummy.com{base}")) {
            base_url
                .join(&value)
                .map_or(value, |joined| joined.path().to_owned())
        } else {
            value
        }
    } else {
        // Prefix only local URLs
        if value.starts_with('/') {
            format!("{versioned_url}{value}")
        } else {
            value
        }
    };
    match node.set_attribute(attr, &new_value) {
        Ok(()) => {}
        Err(_) => {
            tracing::error!("Not able to update node attribute {attr}");
        }
    }
}

/// Updates all URLs in a JSON file according to a historical version.
///
/// If the file is a `manifest.json`, updates `start_url`, `scope`, and `icons`.
/// Otherwise, updates entries in an index JSON using the historical URL prefix.
///
/// # Arguments
///
/// * `json_str` - The JSON content as a string
/// * `path` - File path (used to distinguish manifest from index JSON)
/// * `date` - Historical version date used to construct URL prefixes
///
/// # Returns
///
/// Returns the updated JSON as a string. If parsing fails, the original JSON is returned.
#[must_use]
pub fn update_json_content(json_str: &str, path: &str, date: &str) -> String {
    let result: anyhow::Result<String> = (|| {
        let mut json_value: Value = serde_json::from_str(json_str)?;
        let url_prefix = format!("/_date/{date}");
        if path.contains("manifest") {
            update_manifest_json(&mut json_value, &url_prefix);
        } else {
            update_index_json(&mut json_value, &url_prefix);
        }

        Ok(serde_json::to_string(&json_value)?)
    })();

    match result {
        Ok(updated) => updated,
        Err(err) => {
            tracing::error!("Error while updating json content: {err}");
            json_str.to_owned()
        }
    }
}
/// Updates URLs inside a `manifest.json` object.
///
/// Prefixes the `start_url` and `scope` fields, as well as all `src` fields
/// in the `icons` array, with the provided `url_prefix`.
///
/// # Arguments
///
/// * `manifest` - Mutable reference to the JSON manifest (`serde_json::Value`)
/// * `url_prefix` - Prefix to prepend to all relevant URL fields
///
fn update_manifest_json(manifest: &mut Value, url_prefix: &str) {
    let Some(obj) = manifest.as_object_mut() else {
        return;
    };
    if let Some(start_url) = obj
        .get("start_url")
        .and_then(Value::as_str)
        .map(str::to_owned)
    {
        obj.insert(
            "start_url".into(),
            Value::String(format!("{url_prefix}{start_url}")),
        );
    }
    if let Some(scope) = obj.get("scope").and_then(Value::as_str).map(str::to_owned) {
        obj.insert(
            "scope".into(),
            Value::String(format!("{url_prefix}{scope}")),
        );
    }

    if let Some(icons) = obj.get_mut("icons").and_then(Value::as_array_mut) {
        for icon in icons {
            if let Some(src) = icon.get("src").and_then(Value::as_str).map(str::to_owned) {
                if let Some(mut_icon) = icon.as_object_mut() {
                    mut_icon.insert("src".into(), Value::String(format!("{url_prefix}{src}")));
                }
            }
        }
    }
}

/// Recursively updates URLs inside a JSON index.
///
/// Walks through all entries in the JSON `index` and prefixes relevant
/// fields (`p`, `j`, `dj`, `fh`) with the given `url_prefix`. If an entry
/// has a `c` field (children), the function recurses into them.
///
/// # Arguments
///
/// * `index` - Mutable reference to the JSON index (`serde_json::Value`)
/// * `url_prefix` - String to prefix all relevant URL fields with
fn update_index_json(index: &mut Value, url_prefix: &str) {
    fn update_one_entry(entry: &mut Value, url_prefix: &str) {
        let Some(obj) = entry.as_object_mut() else {
            return;
        };

        let keys: Vec<String> = obj.keys().cloned().collect();
        #[expect(
            clippy::pattern_type_mismatch,
            reason = " Refactoring to avoid this lint would reduce readability without functional benefit"
        )]
        for key in &keys {
            match key.as_str() {
                "p" | "j" | "dj" | "fh" => {
                    if let Some(Value::String(val)) = obj.get(key) {
                        obj.insert(key.clone(), Value::String(format!("{url_prefix}{val}")));
                    }
                }
                "c" => {
                    if let Some(Value::Array(children)) = obj.get_mut("c") {
                        for child in children {
                            update_one_entry(child, url_prefix);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    update_one_entry(index, url_prefix);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_html_transformation(
        html_input: &str,
        expected_output: &str,
        path: &str,
        version_date: &str,
    ) {
        let parser = Parser::default();
        let doc: Document = parser
            .parse_string(expected_output)
            .expect("parse should succeed");
        let options = SaveOptions {
            no_declaration: true,
            as_xml: true,
            ..Default::default()
        };
        let expected_output_fmt = doc.to_string_with_options(options);
        let result = update_doc_urls(html_input, path, version_date);
        let output = result.expect("function should succeed");
        assert_eq!(output, expected_output_fmt);
    }

    #[test]
    fn test_update_element_a_tag() {
        let html_input = r#"
<!DOCTYPE html SYSTEM "about:legacy-compat">
<html lang="en-US" class="no-js">
  <head>
  </head>
  <body>
    <a href="/">smth</a>
    <a href="/test">smth</a>
    <a href="/test/1">smth</a>
    <a href="test/1">smth</a>
  </body>
</html>
        "#;
        let expected_output = r#"
<!DOCTYPE html SYSTEM "about:legacy-compat">
<html lang="en-US" class="no-js">
  <head>
    <meta itemprop="historical-prefix" content="/_date/12-03-2025"/>
  </head>
  <body>
    <a href="/_date/12-03-2025/">smth</a>
    <a href="/_date/12-03-2025/test">smth</a>
    <a href="/_date/12-03-2025/test/1">smth</a>
    <a href="test/1">smth</a>
  </body>
</html>
        "#;
        let version_date = "12-03-2025";
        let path = "/some/test/path/";
        test_html_transformation(html_input, expected_output, path, version_date);
    }

    #[test]
    fn test_update_element_span_tag() {
        let html_input = r#"
<!DOCTYPE html SYSTEM "about:legacy-compat">
<html lang="en-US" class="no-js">
  <head>
  </head>
  <body>
    <span id="/">smth</span>
    <span id="/test">smth</span>
    <span id="/test/1">smth</span>
    <span id="test/1">smth</span>
    <span href="/test/2">smth</span>
  </body>
</html>
        "#;
        let expected_output = r#"
<!DOCTYPE html SYSTEM "about:legacy-compat">
<html lang="en-US" class="no-js">
  <head>
    <meta itemprop="historical-prefix" content="/_date/02-09-2024"/>
  </head>
  <body>
    <span id="/_date/02-09-2024/">smth</span>
    <span id="/_date/02-09-2024/test">smth</span>
    <span id="/_date/02-09-2024/test/1">smth</span>
    <span id="test/1">smth</span>
    <span href="/test/2">smth</span>
  </body>
</html>
        "#;
        let version_date = "02-09-2024";
        let path = "/some/test/path/";
        test_html_transformation(html_input, expected_output, path, version_date);
    }

    #[test]
    fn test_update_element_h_tag() {
        let html_input = r#"
<!DOCTYPE html SYSTEM "about:legacy-compat">
<html lang="en-US" class="no-js">
  <head>
  </head>
  <body>
    <h1 id="/test">Introduction</h1>
    <h2 id="introduction">Introduction</h1>
    <h6 id="/">Introduction</h1>
  </body>
</html>
        "#;
        let expected_output = r#"
<!DOCTYPE html SYSTEM "about:legacy-compat">
<html lang="en-US" class="no-js">
  <head>
    <meta itemprop="historical-prefix" content="/_date/02-09-2024"/>
  </head>
  <body>
    <h1 id="/_date/02-09-2024/test">Introduction</h1>
    <h2 id="introduction">Introduction</h1>
    <h6 id="/_date/02-09-2024/">Introduction</h1>
  </body>
</html>
        "#;
        let version_date = "02-09-2024";
        let path = "/some/test/path/";
        test_html_transformation(html_input, expected_output, path, version_date);
    }

    #[test]
    fn test_update_element_object_tag() {
        let html_input = r#"
<!DOCTYPE html SYSTEM "about:legacy-compat">
<html lang="en-US" class="no-js">
  <head>
  </head>
  <body>
    <object
        data="./documents/example.pdf"
        type="application/pdf"
        width="600"
        height="800">
        <p>
            Your browser does not support embedded PDFs.
            <a href="./documents/example.pdf">Download the PDF</a>.
        </p>
    </object>
    <object
        data="/not/ending/on/dot/pdf"
        type="application/pdf"
        width="600"
        height="800">
        <p>
            Your browser does not support embedded PDFs.
            <a href="/">Download the PDF</a>.
        </p>
    </object>
  </body>
</html>
        "#;
        let expected_output = r#"
<!DOCTYPE html SYSTEM "about:legacy-compat">
<html lang="en-US" class="no-js">
  <head>
    <meta itemprop="historical-prefix" content="/_date/02-09-2024"/>
  </head>
  <body>
    <object
        data="/some/test/documents/example.pdf"
        type="application/pdf"
        width="600"
        height="800">
        <p>
            Your browser does not support embedded PDFs.
            <a href="/some/test/documents/example.pdf">Download the PDF</a>.
        </p>
    </object>
    <object
        data="/_date/02-09-2024/not/ending/on/dot/pdf"
        type="application/pdf"
        width="600"
        height="800">
        <p>
            Your browser does not support embedded PDFs.
            <a href="/_date/02-09-2024/">Download the PDF</a>.
        </p>
    </object>
  </body>
</html>
        "#;
        let version_date = "02-09-2024";
        let path = "/some/test/path/";
        test_html_transformation(html_input, expected_output, path, version_date);
    }

    #[test]
    fn test_meta_tag_update() {
        let html_input = r#"
<!DOCTYPE html SYSTEM "about:legacy-compat">
<html lang="en-US" class="no-js">
  <head>
    <meta property="og:url"
        content="https://law.cityofsanmateo.org/us/ca/cities/san-mateo/ordinances/2020/19" />
    <meta property="og:type" content="article" />
    <meta property="og:title"
        content="Ord. No. 2020-19. Emergency Ordinance – Outdoor Business Operations | City of San Mateo Law Library" />
    <meta itemprop="toc-json" content="/us/ca/cities/san-mateo/ordinances/2020/19/index.json" data-document="href"/>
    <meta itemprop="doc-type" content="document" data-document=""/>
    <meta itemprop="doc-num" content="2020-19"/>
    <meta itemprop="full-html" content="/"/>
  </head>
  <body>
  </body>
</html>
        "#;
        let expected_output = r#"
<!DOCTYPE html SYSTEM "about:legacy-compat">
<html lang="en-US" class="no-js">
  <head>
    <meta property="og:url"
        content="https://law.cityofsanmateo.org/_date/2025-03-04/us/ca/cities/san-mateo/ordinances/2020/19" />
    <meta property="og:type" content="article" />
    <meta property="og:title"
        content="Ord. No. 2020-19. Emergency Ordinance – Outdoor Business Operations | City of San Mateo Law Library | Historical version from March 04, 2025" />
    <meta itemprop="toc-json" content="/_date/2025-03-04/us/ca/cities/san-mateo/ordinances/2020/19/index.json" data-document="href"/>
    <meta itemprop="doc-type" content="document" data-document=""/>
    <meta itemprop="doc-num" content="2020-19"/>
    <meta itemprop="full-html" content="/_date/2025-03-04/"/>
    <meta itemprop="historical-prefix" content="/_date/2025-03-04"/>
  </head>
  <body>
  </body>
</html>
        "#;
        let version_date = "2025-03-04";
        let path = "/us/ca/cities/san-mateo/ordinances/2020/19";
        test_html_transformation(html_input, expected_output, path, version_date);
    }

    #[test]
    fn test_json_update_on_index_json() {
        let json_input = r#"
{
    "p" : "/us/ca/cities/san-mateo/ordinances/2025/2#1",
    "j" : "/us/ca/cities/san-mateo/ordinances/2025/2#2",
    "dj" : "/us/ca/cities/san-mateo/ordinances/2025/2#3",
    "fh" : "/us/ca/cities/san-mateo/ordinances/2025/2#4",
    "q" : "/us/ca/cities/san-mateo/ordinances/2025/2#5",
    "c" : [
        {
            "p" : "/us/ca/cities/san-mateo/ordinances/2025/2#6",
            "j" : "/us/ca/cities/san-mateo/ordinances/2025/2#7",
            "n" : [
                {
                    "p" : "/us/ca/cities/san-mateo/ordinances/2025/2#8"
                }
            ]
        },
        {
            "c": [ 
                {
                    "p" : "/us/ca/cities/san-mateo/ordinances/2025/2#9"
                }
            ]
        }
    ]
}
        "#;
        let expected_output = r#"
{
    "p" : "/_date/2025-03-04/us/ca/cities/san-mateo/ordinances/2025/2#1",
    "j" : "/_date/2025-03-04/us/ca/cities/san-mateo/ordinances/2025/2#2",
    "dj" : "/_date/2025-03-04/us/ca/cities/san-mateo/ordinances/2025/2#3",
    "fh" : "/_date/2025-03-04/us/ca/cities/san-mateo/ordinances/2025/2#4",
    "q" : "/us/ca/cities/san-mateo/ordinances/2025/2#5",
    "c" : [
        {
            "p" : "/_date/2025-03-04/us/ca/cities/san-mateo/ordinances/2025/2#6",
            "j" : "/_date/2025-03-04/us/ca/cities/san-mateo/ordinances/2025/2#7",
            "n" : [
                {
                    "p" : "/us/ca/cities/san-mateo/ordinances/2025/2#8"
                }
            ]
        },
        {
            "c": [ 
                {
                    "p" : "/_date/2025-03-04/us/ca/cities/san-mateo/ordinances/2025/2#9"
                }
            ]
        }
    ]
}        
        "#;
        let version_date = "2025-03-04";
        let path = "/us/ca/cities/san-mateo/ordinances/2020/19/index.json";
        let json_value: Value =
            serde_json::from_str(expected_output).expect("expect to parse json");
        let expected_output_fmt =
            serde_json::to_string(&json_value).expect("expect to convert json to string");
        let output = update_json_content(json_input, path, version_date);
        assert_eq!(output, expected_output_fmt.to_owned());
    }

    #[test]
    fn test_json_update_on_manifest_json() {
        let json_input = r#"
{
  "name": "City of San Mateo Law Library",
  "icons": [
    {
      "src": "/us/ca/cities/san-mateo/_document/v2/images/favicons/android-chrome-48x48.png",
      "sizes": "48x48",
      "type": "image/png"
    },
    {
      "src": "/us/ca/cities/san-mateo/_document/v2/images/favicons/android-chrome-96x96.png",
      "sizes": "96x96",
      "type": "image/png"
    }
  ],
  "display": "standalone",
  "start_url": "/",
  "scope": "/"
}
        "#;
        let expected_output = r#"
{
  "name": "City of San Mateo Law Library",
  "icons": [
    {
      "src": "/_date/2025-03-04/us/ca/cities/san-mateo/_document/v2/images/favicons/android-chrome-48x48.png",
      "sizes": "48x48",
      "type": "image/png"
    },
    {
      "src": "/_date/2025-03-04/us/ca/cities/san-mateo/_document/v2/images/favicons/android-chrome-96x96.png",
      "sizes": "96x96",
      "type": "image/png"
    }
  ],
  "display": "standalone",
  "start_url": "/_date/2025-03-04/",
  "scope": "/_date/2025-03-04/"
}   
        "#;
        let version_date = "2025-03-04";
        let path = "/manifest.json";
        let json_value: Value =
            serde_json::from_str(expected_output).expect("expect to parse json");
        let expected_output_fmt =
            serde_json::to_string(&json_value).expect("expect to convert json to string");
        let output = update_json_content(json_input, path, version_date);
        assert_eq!(output, expected_output_fmt.to_owned());
    }
}
