//! Serve documents in a Stelae archive.
#![allow(clippy::exit)]
#![allow(clippy::unused_async)]
use crate::stelae::archive::Archive;
use crate::utils::archive::get_name_parts;
use crate::utils::git::Repo;
use crate::utils::http::get_contenttype;
use crate::{server::tracing::StelaeRootSpanBuilder, stelae::stele::Stele};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{
    get, guard, web, App, Error, HttpRequest, HttpResponse, HttpServer, Resource, Responder, Route,
    Scope,
};
use git2::Repository;
use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, fmt, path::Path, path::PathBuf};
use tracing_actix_web::TracingLogger;

use actix_http::body::MessageBody;
use actix_service::{
    apply, apply_fn_factory, boxed, IntoServiceFactory, ServiceFactory, ServiceFactoryExt,
    Transform,
};

#[allow(clippy::expect_used)]
/// Remove leading and trailing `/`s from the `path` string.
fn clean_path(path: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?:^/*|/*$)").expect("Failed to compile regex!?!");
    }
    RE.replace_all(path, "").to_string()
}

/// Global, read-only state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Fully initialized Stelae archive
    pub archive: Archive,
}

/// Git repository to serve
struct RepoState {
    /// git2 repository pointing to the repo in the archive.
    repo: Repo,
    ///Latest or historical
    serve: String,
}

/// Shared, read-only app state
pub struct SharedState {
    /// Repository to fall back to if the current one is not found
    fallback: Option<RepoState>,
}

impl fmt::Debug for RepoState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Repo for {} in the archive at {}",
            self.repo.name,
            self.repo.path.display()
        )
    }
}

impl fmt::Debug for SharedState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let &Some(ref fallback) = &self.fallback {
            write!(
                f,
                "Repo for {} in the archive at {}",
                fallback.repo.name,
                fallback.repo.path.display()
            )
        } else {
            write!(f, "No fallback repo")
        }
    }
}

impl Clone for RepoState {
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            serve: self.serve.clone(),
            // fallback: self.fallback.clone(),
        }
    }
}

impl Clone for SharedState {
    fn clone(&self) -> Self {
        Self {
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

/// Serve current document
async fn serve(
    req: HttpRequest,
    shared: web::Data<SharedState>,
    data: web::Data<RepoState>,
) -> impl Responder {
    // dbg!(&data);
    // dbg!(&shared);
    // dbg!(&req.path().to_owned());
    // let mut path = req.path().to_owned();
    // dbg!(&path);
    let mut prefix: String = req
        .match_info()
        .get("prefix")
        .unwrap_or_default()
        .parse()
        .unwrap();
    // dbg!(&prefix);
    let mut tail: String = req.match_info().get("tail").unwrap().parse().unwrap();
    // dbg!(&tail);
    // let (prefix, tail): (String, String) = req.match_info().load().unwrap();
    // dbg!(&prefix);
    // dbg!(&tail);
    let mut path = format!("{}/{}", prefix, tail);
    path = clean_path(&path);
    // dbg!(&path);
    let blob = data.repo.get_bytes_at_path("HEAD", &path);
    let contenttype = get_contenttype(&path);
    format!(
        "{}, {}",
        req.path().to_owned(),
        data.repo.path.to_string_lossy()
    );
    if let Ok(content) = blob {
        HttpResponse::Ok().insert_header(contenttype).body(content)
    } else if let Some(ref fallback) = shared.fallback {
        let blob = fallback.repo.get_bytes_at_path("HEAD", &path);
        blob.map_or_else(
            |_| HttpResponse::BadRequest().into(),
            |content| HttpResponse::Ok().insert_header(contenttype).body(content),
        )
    } else {
        HttpResponse::BadRequest().into()
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
    individual: bool,
) -> std::io::Result<()> {
    let bind = "127.0.0.1";
    let message = "Running Publish Server on a Stelae archive at";
    tracing::info!("{message} '{raw_archive_path}' on http://{bind}:{port}.",);

    let archive = Archive::parse(archive_path, &PathBuf::from(raw_archive_path), individual)
        .unwrap_or_else(|_| {
            tracing::error!("Unable to parse archive at '{raw_archive_path}'.");
            std::process::exit(1);
        });
    let state = AppState { archive };
    let root = state.archive.get_root().unwrap();
    let shared_state = init_shared_app_state(root);
    // let shared_state = SharedState { fallback: None };
    dbg!(&shared_state);

    HttpServer::new(move || init_app(shared_state.clone(), state.clone()))
        .bind((bind, port))?
        .run()
        .await
}

/// Initialize the application
#[must_use]
pub fn init_app(
    shared_state: SharedState,
    state: AppState,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
> {
    App::new().service(
        web::scope("")
            .app_data(web::Data::new(shared_state))
            .wrap(TracingLogger::<StelaeRootSpanBuilder>::new())
            .configure(|cfg| init_routes(cfg, state)),
    )
}

/// Routes
fn init_routes(cfg: &mut web::ServiceConfig, mut state: AppState) {
    let mut scopes: Vec<Scope> = vec![];
    // initialize root stele routes and scopes
    let root = state.archive.get_root().unwrap();
    let mut root_scope: Scope = web::scope("");

    // TODO: this has to get moved to a function
    for stele in state.archive.stelae.values() {
        if let &Some(ref repositories) = &stele.repositories {
            // Root Stele
            let sorted_repositories = repositories.get_sorted_repositories();
            if stele.get_qualified_name() == root.get_qualified_name() {
                for repository in &sorted_repositories {
                    let custom = &repository.custom;
                    let repo_state = {
                        let name = &repository.name;
                        let mut repo_path = state.archive.path.to_string_lossy().into_owned();
                        repo_path = format!("{repo_path}/{name}");
                        RepoState {
                            repo: Repo {
                                archive_path: state.archive.path.to_string_lossy().to_string(),
                                path: PathBuf::from(repo_path.clone()),
                                org: stele.org.clone(),
                                name: name.to_string(),
                                repo: Repository::open(repo_path)
                                    .expect("Unable to open Git repository"),
                            },
                            serve: custom.serve.clone(),
                        }
                    };
                    for route in custom.routes.iter().flat_map(|r| r.iter()) {
                        //ignore routes in child stele that start with underscore
                        if route.starts_with('_') {
                            // TODO: append route to root stele scope
                            continue;
                        }
                        let actix_route = format!("/{{tail:{}}}", &route);
                        root_scope = root_scope.service(
                            web::resource(actix_route.as_str())
                                .route(web::get().to(serve))
                                .app_data(web::Data::new(repo_state.clone())),
                        );
                    }
                    if let &Some(ref underscore_scope) = &custom.scope {
                        let actix_underscore_scope = web::scope(underscore_scope.as_str()).service(
                            web::scope("").service(
                                web::resource("/{tail:.*}")
                                    .route(web::get().to(serve))
                                    .app_data(web::Data::new(repo_state.clone())),
                            ),
                        );
                        scopes.push(actix_underscore_scope);
                    }
                }
                continue;
            }
            //Child Stele
            for scope in repositories.scopes.iter().flat_map(|s| s.iter()) {
                // let escaped_scope = regex::escape(scope);
                // let url_namespace = format!("{{namespace:^{}/.*}}", &scope.as_str());
                // let url_namespace = format!("{{namespace:({})+}}", &scope.as_str());
                // dbg!(&url_namespace);
                // let mut actix_scope = web::scope("/{namespace:us/ca/cities/san-mateo}");
                let scope_str = format!("/{{prefix:{}}}", &scope.as_str());
                let mut actix_scope = web::scope(scope_str.as_str());
                // dbg!(&scope);
                for repository in &sorted_repositories {
                    let custom = &repository.custom;
                    let repo_state = {
                        let name = &repository.name;
                        let mut repo_path = state.archive.path.to_string_lossy().into_owned();
                        repo_path = format!("{repo_path}/{name}");
                        RepoState {
                            repo: Repo {
                                archive_path: state.archive.path.to_string_lossy().to_string(),
                                path: PathBuf::from(repo_path.clone()),
                                org: stele.org.clone(),
                                name: name.to_string(),
                                repo: Repository::open(repo_path)
                                    .expect("Unable to open Git repository"),
                            },
                            serve: custom.serve.clone(),
                        }
                    };
                    for route in custom.routes.iter().flat_map(|r| r.iter()) {
                        //ignore routes in child stele that start with underscore
                        if route.starts_with('_') {
                            // TODO: append route to root stele scope
                            continue;
                        }
                        let actix_route = format!("/{{tail:{}}}", &route);
                        actix_scope = actix_scope.service(
                            web::resource(actix_route.as_str())
                                .route(web::get().to(serve))
                                .app_data(web::Data::new(repo_state.clone())),
                        );
                    }
                    // TODO: This is not necessary in child Stele
                    // if let &Some(ref underscore_scope) = &custom.scope {
                    //     let actix_underscore_scope = web::scope(underscore_scope.as_str()).service(
                    //         web::scope(scope.as_str()).service(
                    //             web::resource("/{tail:.*}")
                    //                 .route(web::get().to(serve))
                    //                 .app_data(web::Data::new(repo_state.clone())),
                    //         ),
                    //     );
                    //     scopes.push(actix_underscore_scope);
                    // }
                }
                scopes.push(actix_scope);
            }
        }
    }
    for scope in scopes {
        cfg.service(scope);
    }
    // Register root stele scope last
    cfg.service(root_scope);
}

/// Initialize the shared application state
/// Currently shared application state consists of:
///     - fallback: used as a data repository to resolve data when no other url matches the request
/// # Returns
/// Returns a `SharedState` object
/// # Panics
/// Will panic if `get_name_parts` or `Repo::new` fails
#[must_use]
pub fn init_shared_app_state(root: &Stele) -> SharedState {
    let fallback = root.get_fallback_repo().map(|repo| {
        let (org, name) = get_name_parts(&repo.name).unwrap();
        RepoState {
            repo: Repo::new(&root.archive_path, &org, &name).unwrap(),
            serve: repo.custom.serve,
        }
    });
    SharedState { fallback }
}
