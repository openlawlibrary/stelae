//! Legacy git microserver.

#![allow(
    // Will be deprecated upon completion of Publish Server migration to rust.
    clippy::exhaustive_structs,
    // Unused asyncs are the norm in Actix route definition files
    clippy::unused_async
)]

use crate::utils::git::Repo;
use crate::utils::http::get_contenttype;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use lazy_static::lazy_static;
use regex::Regex;
use std::path::PathBuf;

/// Global, read-only state passed into the actix app
struct AppState {
    /// path to the Stele library
    library_path: PathBuf,
}

#[allow(clippy::expect_used)]
/// Remove leading and trailing `/`s from the `path` string.
fn clean_path(path: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?:^/*|/*$)").expect("Failed to compile regex!?!");
    }
    RE.replace_all(path, "").to_string()
}

/// Return the content in the stele library in the `{namespace}/{name}`
/// repo at the `commitish` commit at the `remainder` path.
/// Return 404 if any are not found or there are any errors.
#[get("/{namespace}/{name}/{commitish}{remainder:/+([^{}]*?)?/*}")]
async fn get_blob(
    path: web::Path<(String, String, String, String)>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (namespace, name, commitish, remainder) = path.into_inner();
    let lib_path = &data.library_path;
    let repo = match Repo::new(lib_path, &namespace, &name) {
        Ok(repo) => repo,
        Err(_e) => {
            return HttpResponse::NotFound().body(format!("repo {namespace}/{name} does not exist"))
        }
    };
    let blob_path = clean_path(&remainder);
    let contenttype = get_contenttype(&blob_path);

    match repo.get_bytes_at_path(&commitish, &blob_path) {
        Ok(content) => HttpResponse::Ok().insert_header(contenttype).body(content),
        Err(_e) => HttpResponse::NotFound().body(format!(
            "content at {remainder} for {commitish} in repo {namespace}/{name} does not exist"
        )),
    }
}

/// Serve git repositories in the Stele library.
#[actix_web::main] // or #[tokio::main]
#[allow(clippy::print_stdout)]
pub async fn serve_git(
    raw_library_path: &str,
    library_path: PathBuf,
    port: u16,
) -> std::io::Result<()> {
    println!(
        "Serving content from the Stele library at {} on http://127.0.0.1:{}.",
        raw_library_path, port
    );

    HttpServer::new(move || {
        App::new()
            .service(get_blob)
            .app_data(web::Data::new(AppState {
                library_path: library_path.clone(),
            }))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
