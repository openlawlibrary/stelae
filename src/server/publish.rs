//! Serve documents in a Stelae archive.
#![allow(clippy::exit)]
#![allow(clippy::unused_async)]
use crate::server::tracing::StelaeRootSpanBuilder;
use crate::stelae::archive::Archive;
use actix_web::{get, web, App, HttpRequest, HttpServer, Resource, Route, Scope};
use git2::Repository;
use std::{collections::HashMap, fmt, path::Path, path::PathBuf};
use tracing_actix_web::TracingLogger;
/// Global, read-only state
#[derive(Debug, Clone)]
struct AppState {
    /// Fully initialized Stelae archive
    archive: Archive,
}

struct RepoState {
    /// Path to Stele
    path: PathBuf,
    /// Repo org
    org: String,
    /// Repo name
    name: String,
    /// git2 repository pointing to the repo in the archive.
    repo: Repository,
    ///Latest or historical
    serve: String,
    ///Fallback Repository if this one is not found
    fallback: Option<Box<RepoState>>,
}

impl fmt::Debug for RepoState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Repo for {} in the archive at {}",
            self.name,
            self.path.display()
        )
    }
}

impl Clone for RepoState {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            org: self.org.clone(),
            name: self.name.clone(),
            repo: Repository::open(self.path.clone()).unwrap(),
            serve: self.serve.clone(),
            fallback: self.fallback.clone(),
        }
    }
}

/// Index path for testing purposes
// #[get("/t")]
async fn index() -> &'static str {
    "Welcome to Publish Server"
}

async fn default() -> &'static str {
    "Default"
}

async fn serve(req: HttpRequest, data: web::Data<RepoState>) -> String {
    dbg!(&data);
    format!("{}, {}", req.path().to_owned(), data.path.to_string_lossy());
    let repo = data.repo.clone();
    let path = data.path.clone();
    let commitish = data.commitish.clone();
    let blob = find_blob(&repo, &path, &commitish);
    let contenttype = get_contenttype(&path);
    match blob {
        Ok(content) => HttpResponse::Ok().insert_header(contenttype).body(content),
        Err(_) => HttpResponse::NotFound().body(GIT_REQUEST_NOT_FOUND),
    }
}

/// Index path for testing purposes
// #[get("/test")]
async fn test(req: HttpRequest, data: web::Data<HashMap<String, String>>) -> String {
    format!(
        "{}, {}",
        req.path().to_owned(),
        data.get("cityofsanmateo")
            .unwrap_or(&("no value").to_owned())
    )
}

/// Serve documents in a Stelae archive.
#[actix_web::main]
pub async fn serve_archive(
    raw_archive_path: &str,
    archive_path: PathBuf,
    port: u16,
) -> std::io::Result<()> {
    let bind = "127.0.0.1";
    let message = "Running Publish Server on a Stelae archive at";
    tracing::info!("{message} '{raw_archive_path}' on http://{bind}:{port}.",);

    let archive =
        Archive::parse(archive_path, PathBuf::from(raw_archive_path)).unwrap_or_else(|_| {
            tracing::error!("Unable to parse archive at '{raw_archive_path}'.");
            std::process::exit(1);
        });
    let mut state = AppState { archive };
    //TODO: root stele is a stele from which we began serving the archive
    // let root = state.archive.get_root().unwrap_or_else(|_| {
    //     tracing::error!("Unable to determine root Stele.");
    //     std::process::exit(1);
    // });

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::<StelaeRootSpanBuilder>::new())
            .app_data(web::Data::new(state.clone()))
            .configure(|cfg| init_routes(cfg, state.clone()))
    })
    .bind((bind, port))?
    .run()
    .await
}

/// Routes
fn init_routes(cfg: &mut web::ServiceConfig, state: AppState) {
    let mut scopes: Vec<Scope> = vec![];
    // initialize root stele routes and scopes
    for stele in state.archive.stelae.values() {
        if let Some(repositories) = &stele.repositories {
            for scope in repositories.scopes.iter().flat_map(|s| s.iter()) {
                dbg!(&scope);
                let mut actix_scope = web::scope(scope.as_str());
                for (name, repository) in &repositories.repositories {
                    let custom = &repository.custom;
                    let repo_state = {
                        let mut repo_path = stele
                            .path
                            .clone()
                            .parent()
                            .unwrap()
                            .to_string_lossy()
                            .into_owned();
                        repo_path = format!("{repo_path}/{name}");
                        RepoState {
                            path: PathBuf::from(repo_path.clone()),
                            org: stele.org.clone(),
                            name: name.to_string(),
                            repo: Repository::open(repo_path)
                                .expect("Unable to open Git repository"),
                            serve: custom.serve.clone(),
                            fallback: None,
                        }
                    };
                    for route in custom.routes.iter().flat_map(|r| r.iter()) {
                        //ignore routes in child stele that start with underscore
                        if route.starts_with("~ _") {
                            // TODO: append route to root stele scope
                            continue;
                        }
                        let actix_route = format!("/{{prefix:{}}}", &route);
                        actix_scope = actix_scope.service(
                            web::resource(actix_route.as_str())
                                .route(web::get().to(serve))
                                .app_data(web::Data::new(repo_state.clone())),
                        );
                    }
                }
                scopes.push(actix_scope);
            }
        }
    }
    for scope in scopes {
        cfg.service(scope);
    }
    // {
    //     let mut smc_hashmap = HashMap::new();
    //     smc_hashmap.insert("cityofsanmateo".to_owned(), "some value for SMC".to_owned());
    //     let mut dc_hashmap = HashMap::new();
    //     dc_hashmap.insert("dc".to_owned(), "some value for DC".to_owned());

    //     cfg.service(
    //         web::scope("/us/ca/cities/san-mateo")
    //             .service(web::resource("/{prefix:_reader/.*}")
    //             // .route("/{prefix:_reader/.*}", web::get().to(test))
    //             // .app_data(web::Data::new(smc_hashmap))
    //             .route(web::get().to(test)))
    //             // .route("/{pdfs:.*/.*pdf}", web::get().to(test))
    //             // .app_data(web::Data::new(dc_hashmap))
    //             .service(web::resource("/{pdfs:.*/.*pdf}").route(web::get().to(test))), // .service(index)
    //                                                                                              // .service(test),
    //     ).app_data(web::Data::new(smc_hashmap.clone()));

    //     let mut scope = web::scope("/congress");

    //     scope = scope.service(web::resource("/{prefix:_reader/.*}").route(web::get().to(test)));
    //     scope = scope.service(web::resource("/{pdfs:.*/.*pdf}").route(web::get().to(test)));
    //     scope = scope.app_data(web::Data::new(smc_hashmap.clone()));
    //     scope = scope.app_data(web::Data::new(dc_hashmap.clone()));
    //     cfg.service(scope);

    //     cfg.service(web::scope("/fedlaws")
    //         .service(web::resource("/{prefix:_reader/.*}")
    //         .route(web::get().to(test)))
    //         // .app_data(web::Data::new(smc_hashmap.clone()))
    //         .service(web::resource("/{pdfs:.*/.*pdf}").route(web::get().to(test))
    //         .app_data(web::Data::new(smc_hashmap)))
    //     );
    // }
    // {
    //     let mut dc_hashmap = HashMap::new();
    //     dc_hashmap.insert("dc".to_owned(), "some value for DC".to_owned());

    //     cfg.service(
    //         web::scope("/us/dc").app_data(web::Data::new(dc_hashmap)), // .service(test),
    //     );
    // }
}
