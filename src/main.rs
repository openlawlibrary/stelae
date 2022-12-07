//! Stele commandline.
#![allow(clippy::self_named_module_files)]
#![allow(clippy::std_instead_of_alloc)]
#![allow(clippy::implicit_return)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::exhaustive_structs)]

use std::path::{Path, PathBuf};

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use lazy_static::lazy_static;
use regex::Regex;
// use std::env::current_dir;
use stele::utils::git::Repo;
use stele::utils::library::find_library_path;

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
#[allow(clippy::unused_async)]
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

    match repo.get_bytes_at_path(&commitish, &blob_path) {
        Ok(content) => HttpResponse::Ok().body(content),
        Err(_e) => HttpResponse::NotFound().body(format!(
            "content at {remainder} for {commitish} in repo {namespace}/{name} does not exist"
        )),
    }
}

/// Stele is currently just a simple git server.
/// run from the library directory or pass
/// path to library.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the Stele library. Defaults to cwd.
    #[arg(short, long, default_value_t = String::from(".").to_owned())]
    library_path: String,
    /// Port on which to serve the library.
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

/// Global, read-only state passed into the actix app
struct AppState {
    /// path to the Stele library
    library_path: PathBuf,
}

#[actix_web::main] // or #[tokio::main]
#[allow(clippy::print_stdout)]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let library_path_wd = Path::new(&cli.library_path);
    let library_path = if let Ok(lpath) = find_library_path(library_path_wd) {
        lpath
    } else {
        println!(
            "error: could not find `.stele` folder in `{}` or any parent directory",
            &cli.library_path
        );
        std::process::exit(1);
    };

    // .unwrap_or_else(|_| panic!("Unable to find a library at {}", &cli.library_path));

    println!(
        "Serving content from the Stele library at {} on port http://127.0.0.1:{}.",
        &cli.library_path, &cli.port
    );

    HttpServer::new(move || {
        App::new()
            .service(get_blob)
            .app_data(web::Data::new(AppState {
                library_path: library_path.clone(),
            }))
    })
    .bind(("127.0.0.1", cli.port))?
    .run()
    .await
}
