// use std::str::Bytes;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use stele::utils::git::Repo;
// use std::path::Path;
// use std::ffi::OsStr;

// fn get_extension_from_filename(filename: &str) -> Option<&str> {
//     Path::new(filename)
//         .extension()
//         .and_then(OsStr::to_str)}

fn clean_path(path: &str) -> &str {
    let start = usize::from(path.starts_with('/'));
    let end = if path.len() > 1 && path.ends_with('/') {
        path.len() - 1
    } else {
        path.len()
    };
    &path[start..end]
}

#[get("/{namespace}/{name}/{commitish}{remainder:(/[^{}]*)?}")]
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
    let blob_path: Vec<&str> = clean_path(&remainder).split('/').collect();

    match repo.get_bytes_at_path(&commitish, &blob_path) {
        Ok(content) => HttpResponse::Ok().body(content),
        Err(_e) => HttpResponse::NotFound().body(format!(
            "content at {remainder} for {commitish} in repo {namespace}/{name} does not exist"
        )),
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    library_path: String,
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

struct AppState {
    library_path: String,
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    HttpServer::new(move || {
        App::new()
            .service(get_blob)
            .app_data(web::Data::new(AppState {
                library_path: cli.library_path.clone(),
            }))
    })
    .bind(("127.0.0.1", cli.port))?
    .run()
    .await
}
